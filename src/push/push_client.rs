//! PushClient - TCP+TLS push client
//!
//! Manages a raw TCP+TLS connection to the push server for receiving
//! real-time market data and account push notifications.
//! Supports connection authentication, subscribe/unsubscribe, callbacks,
//! heartbeat keep-alive, and automatic reconnection.
//! Uses Protobuf binary protocol with varint32 length-prefix framing.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Notify};

use crate::config::ClientConfig;
use crate::signer::sign_with_rsa;
use super::callbacks::Callbacks;
use super::pb;
use super::pb::socket_common::{Command, DataType};
use super::pb::push_data::Body;
use super::proto_message;
use super::push_message::SubjectType;
use super::varint::{encode_varint32, decode_varint32};

/// Default push server address (raw TCP + TLS)
const DEFAULT_PUSH_URL: &str = "openapi.tigerfintech.com:9883";
/// Default heartbeat interval in seconds
const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 10;
/// Default reconnect interval in seconds
const DEFAULT_RECONNECT_INTERVAL_SECS: u64 = 5;
/// Maximum reconnect interval in seconds
const MAX_RECONNECT_INTERVAL_SECS: u64 = 60;
/// Default connect timeout in seconds
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;
/// SDK version string
const SDK_VERSION: &str = "rust-sdk/1.0.0";
/// Protocol version
const ACCEPT_VERSION: &str = "2";
/// Default heartbeat send interval in milliseconds
const DEFAULT_SEND_INTERVAL: u32 = 10000;
/// Default heartbeat receive interval in milliseconds
const DEFAULT_RECEIVE_INTERVAL: u32 = 10000;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

/// PushClient configuration options
#[derive(Default)]
pub struct PushClientOptions {
    pub push_url: Option<String>,
    pub heartbeat_interval_secs: Option<u64>,
    pub reconnect_interval_secs: Option<u64>,
    pub auto_reconnect: Option<bool>,
    pub connect_timeout_secs: Option<u64>,
}

/// TCP+TLS push client
///
/// Internally manages the TLS connection, read loop, heartbeat loop,
/// and reconnection logic. The user only needs to call `connect()`,
/// set callbacks, and subscribe/unsubscribe.
pub struct PushClient {
    config: ClientConfig,
    push_url: String,
    heartbeat_interval: Duration,
    reconnect_interval: Duration,
    connect_timeout: Duration,
    auto_reconnect: bool,
    state: Arc<RwLock<ConnectionState>>,
    callbacks: Arc<RwLock<Callbacks>>,
    /// Market data subscriptions: subject -> symbols set
    subscriptions: Arc<RwLock<HashMap<SubjectType, HashSet<String>>>>,
    /// Account-level subscriptions
    account_subs: Arc<RwLock<HashSet<SubjectType>>>,
    /// Channel sender for writing framed messages to the TLS stream
    write_tx: Arc<RwLock<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    /// Notification for CONNECTED response received
    connected_notify: Arc<Notify>,
    /// Stop signal for background tasks
    stop_tx: Arc<RwLock<Option<tokio::sync::broadcast::Sender<()>>>>,
}

impl PushClient {
    /// Create a new push client
    pub fn new(config: ClientConfig, options: Option<PushClientOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            config,
            push_url: opts.push_url.unwrap_or_else(|| DEFAULT_PUSH_URL.into()),
            heartbeat_interval: Duration::from_secs(
                opts.heartbeat_interval_secs.unwrap_or(DEFAULT_HEARTBEAT_INTERVAL_SECS),
            ),
            reconnect_interval: Duration::from_secs(
                opts.reconnect_interval_secs.unwrap_or(DEFAULT_RECONNECT_INTERVAL_SECS),
            ),
            connect_timeout: Duration::from_secs(
                opts.connect_timeout_secs.unwrap_or(DEFAULT_CONNECT_TIMEOUT_SECS),
            ),
            auto_reconnect: opts.auto_reconnect.unwrap_or(true),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            callbacks: Arc::new(RwLock::new(Callbacks::default())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            account_subs: Arc::new(RwLock::new(HashSet::new())),
            write_tx: Arc::new(RwLock::new(None)),
            connected_notify: Arc::new(Notify::new()),
            stop_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// Get current connection state
    pub fn state(&self) -> ConnectionState {
        *self.state.read().unwrap()
    }

    /// Set callback functions
    pub fn set_callbacks(&self, cb: Callbacks) {
        *self.callbacks.write().unwrap() = cb;
    }

    // ===== Message sending =====

    /// Send a Protobuf Request message (varint32 + protobuf binary frame)
    fn send_request(&self, request: &pb::Request) -> bool {
        let tx_guard = self.write_tx.read().unwrap();
        if let Some(tx) = tx_guard.as_ref() {
            let proto_bytes = request.encode_to_vec();
            let framed = encode_varint32(&proto_bytes);
            tx.send(framed).is_ok()
        } else {
            false
        }
    }

    // ===== Heartbeat =====

    /// Send a heartbeat message
    pub fn send_heartbeat(&self) -> bool {
        let request = proto_message::build_heartbeat_message();
        self.send_request(&request)
    }

    // ===== Disconnect =====

    /// Disconnect: send DISCONNECT request, then close the connection
    pub fn disconnect(&self) {
        // Send DISCONNECT request (best effort)
        let request = proto_message::build_disconnect_message();
        self.send_request(&request);

        *self.state.write().unwrap() = ConnectionState::Disconnected;

        // Stop background tasks
        if let Some(tx) = self.stop_tx.read().unwrap().as_ref() {
            let _ = tx.send(());
        }
        *self.write_tx.write().unwrap() = None;
        *self.stop_tx.write().unwrap() = None;

        let cbs = self.callbacks.read().unwrap();
        if let Some(cb) = &cbs.on_disconnect {
            cb();
        }
    }

    // ===== Subscribe / Unsubscribe =====

    /// Send a subscribe request
    pub fn subscribe(
        &self,
        subject: &SubjectType,
        symbols: Option<&str>,
        account: Option<&str>,
        market: Option<&str>,
    ) -> bool {
        let data_type = proto_message::subject_to_data_type(subject);
        let request = proto_message::build_subscribe_message(data_type, symbols, account, market);
        self.send_request(&request)
    }

    /// Send an unsubscribe request
    pub fn unsubscribe(
        &self,
        subject: &SubjectType,
        symbols: Option<&str>,
        account: Option<&str>,
        market: Option<&str>,
    ) -> bool {
        let data_type = proto_message::subject_to_data_type(subject);
        let request =
            proto_message::build_unsubscribe_message(data_type, symbols, account, market);
        self.send_request(&request)
    }

    // ===== Message handling =====

    /// Handle a received binary message (varint32 + protobuf frame).
    ///
    /// This is the public entry point used by tests and the legacy example.
    /// It decodes the varint32 frame, deserializes the Response, and dispatches.
    pub fn handle_message(&self, data: &[u8]) {
        // Decode varint32 frame
        let (proto_bytes, _remaining) = match decode_varint32(data) {
            Some(result) => result,
            None => {
                let cbs = self.callbacks.read().unwrap();
                if let Some(cb) = &cbs.on_error {
                    cb("varint32 frame decode failed".to_string());
                }
                return;
            }
        };

        // Deserialize to Response
        let response: pb::Response = match pb::Response::decode(proto_bytes) {
            Ok(r) => r,
            Err(_) => {
                let cbs = self.callbacks.read().unwrap();
                if let Some(cb) = &cbs.on_error {
                    cb("protobuf deserialization failed".to_string());
                }
                return;
            }
        };

        self.dispatch_response(&response);
    }

    /// Dispatch a deserialized Response to the appropriate callback
    fn dispatch_response(&self, response: &pb::Response) {
        let cbs = self.callbacks.read().unwrap();

        if response.command == Command::Connected as i32 {
            // CONNECTED -> mark connection successful
            *self.state.write().unwrap() = ConnectionState::Connected;
            // Notify connect() that authentication is complete
            self.connected_notify.notify_one();
            if let Some(cb) = &cbs.on_connect {
                cb();
            }
        } else if response.command == Command::Heartbeat as i32 {
            // HEARTBEAT -> ignore
        } else if response.command == Command::Message as i32 {
            // MESSAGE -> extract PushData and dispatch
            if let Some(push_data) = &response.body {
                self.dispatch_push_data(&cbs, push_data.clone());
            }
        } else if response.command == Command::Error as i32 {
            // ERROR -> trigger on_error or on_kickout callback
            let msg = response.msg.clone().unwrap_or_default();
            if msg.contains("kickout") || msg.contains("kick") {
                if let Some(cb) = &cbs.on_kickout {
                    cb(msg);
                }
            } else {
                if let Some(cb) = &cbs.on_error {
                    cb(format!("服务端错误: {}", msg));
                }
            }
        } else if response.command == Command::Disconnect as i32 {
            // DISCONNECT -> trigger on_disconnect callback
            if let Some(cb) = &cbs.on_disconnect {
                cb();
            }
        }
    }

    /// Dispatch PushData to the corresponding callback based on data_type and body
    fn dispatch_push_data(&self, cbs: &Callbacks, push_data: pb::PushData) {
        let data_type = push_data.data_type;
        let body = match push_data.body {
            Some(b) => b,
            None => {
                if let Some(cb) = &cbs.on_error {
                    cb("PushData body is empty".to_string());
                }
                return;
            }
        };

        match body {
            Body::QuoteData(d) => {
                if data_type == DataType::Quote as i32 {
                    if let Some(cb) = &cbs.on_quote {
                        cb(d);
                    }
                } else if data_type == DataType::Option as i32 {
                    if let Some(cb) = &cbs.on_option {
                        cb(d);
                    }
                } else if data_type == DataType::Future as i32 {
                    if let Some(cb) = &cbs.on_future {
                        cb(d);
                    }
                } else {
                    // QuoteBbo also uses QuoteData
                    if let Some(cb) = &cbs.on_quote_bbo {
                        cb(d);
                    }
                }
            }
            Body::QuoteDepthData(d) => {
                if let Some(cb) = &cbs.on_depth {
                    cb(d);
                }
            }
            Body::TradeTickData(d) => {
                if let Some(cb) = &cbs.on_tick {
                    cb(d);
                }
            }
            Body::PositionData(d) => {
                if let Some(cb) = &cbs.on_position {
                    cb(d);
                }
            }
            Body::AssetData(d) => {
                if let Some(cb) = &cbs.on_asset {
                    cb(d);
                }
            }
            Body::OrderStatusData(d) => {
                if let Some(cb) = &cbs.on_order {
                    cb(d);
                }
            }
            Body::OrderTransactionData(d) => {
                if let Some(cb) = &cbs.on_transaction {
                    cb(d);
                }
            }
            Body::StockTopData(d) => {
                if let Some(cb) = &cbs.on_stock_top {
                    cb(d);
                }
            }
            Body::OptionTopData(d) => {
                if let Some(cb) = &cbs.on_option_top {
                    cb(d);
                }
            }
            Body::KlineData(d) => {
                if let Some(cb) = &cbs.on_kline {
                    cb(d);
                }
            }
            Body::TickData(d) => {
                if let Some(cb) = &cbs.on_full_tick {
                    cb(d);
                }
            }
        }
    }

    // ===== Subscription state management =====

    /// Add subscription record
    pub fn add_subscription(&self, subject: SubjectType, symbols: &[String]) {
        let mut subs = self.subscriptions.write().unwrap();
        let set = subs.entry(subject).or_insert_with(HashSet::new);
        for s in symbols {
            set.insert(s.clone());
        }
    }

    /// Remove subscription record
    pub fn remove_subscription(&self, subject: SubjectType, symbols: Option<&[String]>) {
        let mut subs = self.subscriptions.write().unwrap();
        match symbols {
            None => {
                subs.remove(&subject);
            }
            Some(syms) => {
                if let Some(set) = subs.get_mut(&subject) {
                    for s in syms {
                        set.remove(s);
                    }
                    if set.is_empty() {
                        subs.remove(&subject);
                    }
                }
            }
        }
    }

    /// Get current market data subscriptions
    pub fn get_subscriptions(&self) -> HashMap<SubjectType, Vec<String>> {
        let subs = self.subscriptions.read().unwrap();
        subs.iter()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
            .collect()
    }

    /// Add account subscription
    pub fn add_account_sub(&self, subject: SubjectType) {
        self.account_subs.write().unwrap().insert(subject);
    }

    /// Remove account subscription
    pub fn remove_account_sub(&self, subject: &SubjectType) {
        self.account_subs.write().unwrap().remove(subject);
    }

    /// Get account-level subscriptions
    pub fn get_account_subscriptions(&self) -> Vec<SubjectType> {
        self.account_subs.read().unwrap().iter().cloned().collect()
    }
}

// ===== Free-standing async functions to avoid async cycle issues =====

/// Connect to the push server, authenticate, and start background tasks.
///
/// This function:
/// 1. Establishes a TCP+TLS connection
/// 2. Spawns a write loop (channel -> TLS stream)
/// 3. Spawns a read loop (TLS stream -> message dispatch)
/// 4. Sends CONNECT authentication message
/// 5. Waits for CONNECTED response
/// 6. Spawns heartbeat loop
pub async fn connect(client: &Arc<PushClient>) -> Result<(), String> {
    {
        let current_state = *client.state.read().unwrap();
        if current_state != ConnectionState::Disconnected {
            return Err("client is already connected or connecting".into());
        }
        *client.state.write().unwrap() = ConnectionState::Connecting;
    }

    // Create stop channel
    let (stop_tx, _) = tokio::sync::broadcast::channel::<()>(1);
    *client.stop_tx.write().unwrap() = Some(stop_tx.clone());

    // Establish TCP connection
    let tcp_stream = match tokio::time::timeout(
        client.connect_timeout,
        tokio::net::TcpStream::connect(&client.push_url),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            *client.state.write().unwrap() = ConnectionState::Disconnected;
            return Err(format!("TCP connection failed: {}", e));
        }
        Err(_) => {
            *client.state.write().unwrap() = ConnectionState::Disconnected;
            return Err("TCP connection timed out".into());
        }
    };

    // TLS handshake (skip hostname verification, same as Go SDK)
    let tls_connector = build_tls_connector();
    let host = client
        .push_url
        .split(':')
        .next()
        .unwrap_or("openapi.tigerfintech.com");
    let server_name = rustls::ServerName::try_from(host)
        .map_err(|e| format!("invalid server name: {}", e))?;

    let tls_stream = match tls_connector.connect(server_name, tcp_stream).await {
        Ok(stream) => stream,
        Err(e) => {
            *client.state.write().unwrap() = ConnectionState::Disconnected;
            return Err(format!("TLS handshake failed: {}", e));
        }
    };

    // Split TLS stream into read and write halves
    let (read_half, write_half) = tokio::io::split(tls_stream);

    // Create write channel
    let (write_tx, write_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    *client.write_tx.write().unwrap() = Some(write_tx);

    // Spawn write loop
    let mut stop_rx_write = stop_tx.subscribe();
    tokio::spawn(async move {
        write_loop(write_half, write_rx, &mut stop_rx_write).await;
    });

    // Spawn read loop
    let client_read = Arc::clone(client);
    let mut stop_rx_read = stop_tx.subscribe();
    tokio::spawn(async move {
        read_loop(&client_read, read_half, &mut stop_rx_read).await;
    });

    // Sign and send CONNECT authentication message
    let sign = sign_with_rsa(&client.config.private_key, &client.config.tiger_id)
        .map_err(|e| format!("RSA signing failed: {}", e))?;

    let connect_req = proto_message::build_connect_message(
        &client.config.tiger_id,
        &sign,
        SDK_VERSION,
        ACCEPT_VERSION,
        DEFAULT_SEND_INTERVAL,
        DEFAULT_RECEIVE_INTERVAL,
        false,
    );
    if !client.send_request(&connect_req) {
        *client.state.write().unwrap() = ConnectionState::Disconnected;
        return Err("failed to send CONNECT message".into());
    }

    // Wait for CONNECTED response
    let notify = client.connected_notify.clone();
    match tokio::time::timeout(client.connect_timeout, notify.notified()).await {
        Ok(_) => {
            // CONNECTED received, state already set by dispatch_response
        }
        Err(_) => {
            *client.state.write().unwrap() = ConnectionState::Disconnected;
            if let Some(tx) = client.stop_tx.read().unwrap().as_ref() {
                let _ = tx.send(());
            }
            *client.write_tx.write().unwrap() = None;
            return Err("timed out waiting for CONNECTED response".into());
        }
    }

    // Spawn heartbeat loop
    let client_hb = Arc::clone(client);
    let mut stop_rx_hb = stop_tx.subscribe();
    tokio::spawn(async move {
        heartbeat_loop(&client_hb, &mut stop_rx_hb).await;
    });

    Ok(())
}

/// Write loop: reads framed messages from the channel and writes them to the TLS stream
async fn write_loop(
    mut writer: tokio::io::WriteHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    stop_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(data) => {
                        if writer.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                    None => break, // channel closed
                }
            }
            _ = stop_rx.recv() => break,
        }
    }
}

/// Read loop: reads from TLS stream, buffers data, decodes varint32 frames,
/// deserializes Response, and dispatches via dispatch_response
async fn read_loop(
    client: &Arc<PushClient>,
    mut reader: tokio::io::ReadHalf<tokio_rustls::client::TlsStream<tokio::net::TcpStream>>,
    stop_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
    let mut buf = vec![0u8; 4096];
    let mut buffer = Vec::new();

    loop {
        tokio::select! {
            result = reader.read(&mut buf) => {
                match result {
                    Ok(0) | Err(_) => {
                        // Connection closed or error
                        let cbs = client.callbacks.read().unwrap();
                        if let Some(cb) = &cbs.on_error {
                            cb("connection closed".to_string());
                        }
                        drop(cbs);

                        // Trigger auto-reconnect if enabled
                        if client.auto_reconnect {
                            let client_clone = Arc::clone(client);
                            tokio::spawn(reconnect_loop(client_clone));
                        }
                        return;
                    }
                    Ok(n) => {
                        buffer.extend_from_slice(&buf[..n]);

                        // Decode all complete varint32 frames in the buffer
                        loop {
                            match decode_varint32(&buffer) {
                                Some((msg, remaining)) => {
                                    match pb::Response::decode(msg) {
                                        Ok(response) => {
                                            client.dispatch_response(&response);
                                        }
                                        Err(_) => {
                                            let cbs = client.callbacks.read().unwrap();
                                            if let Some(cb) = &cbs.on_error {
                                                cb("protobuf deserialization failed".to_string());
                                            }
                                        }
                                    }
                                    buffer = remaining.to_vec();
                                }
                                None => break, // incomplete frame, wait for more data
                            }
                        }
                    }
                }
            }
            _ = stop_rx.recv() => return,
        }
    }
}

/// Heartbeat loop: periodically sends HEARTBEAT messages
async fn heartbeat_loop(
    client: &Arc<PushClient>,
    stop_rx: &mut tokio::sync::broadcast::Receiver<()>,
) {
    let mut interval = tokio::time::interval(client.heartbeat_interval);
    // Skip the first immediate tick
    interval.tick().await;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if !client.send_heartbeat() {
                    return;
                }
            }
            _ = stop_rx.recv() => return,
        }
    }
}

/// Reconnect with exponential backoff.
///
/// Uses `Box::pin` to break the async type cycle:
/// `connect` -> `read_loop` -> `reconnect_loop` -> `connect`
fn reconnect_loop(
    client: Arc<PushClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> {
    Box::pin(async move {
        // Clean up current state
        {
            *client.state.write().unwrap() = ConnectionState::Disconnected;
            if let Some(tx) = client.stop_tx.read().unwrap().as_ref() {
                let _ = tx.send(());
            }
            *client.write_tx.write().unwrap() = None;
            *client.stop_tx.write().unwrap() = None;
        }

        let mut interval = client.reconnect_interval;
        loop {
            tokio::time::sleep(interval).await;

            match connect(&client).await {
                Ok(_) => {
                    // Reconnected successfully, restore subscriptions
                    resubscribe(&client);
                    return;
                }
                Err(_) => {
                    // Exponential backoff
                    interval = interval * 2;
                    let max = Duration::from_secs(MAX_RECONNECT_INTERVAL_SECS);
                    if interval > max {
                        interval = max;
                    }
                }
            }
        }
    })
}

/// Restore subscriptions after reconnection
fn resubscribe(client: &Arc<PushClient>) {
    let subs = client.subscriptions.read().unwrap().clone();
    let acct_subs = client.account_subs.read().unwrap().clone();

    for (subject, symbols) in &subs {
        let symbols_str = symbols.iter().cloned().collect::<Vec<_>>().join(",");
        client.subscribe(subject, Some(&symbols_str), None, None);
    }

    for subject in &acct_subs {
        client.subscribe(subject, None, Some(&client.config.account), None);
    }
}

/// Build a TLS connector with hostname verification skipped
/// (matches the Go SDK's InsecureSkipVerify behavior)
fn build_tls_connector() -> tokio_rustls::TlsConnector {
    use rustls::client::{ServerCertVerified, ServerCertVerifier};
    use rustls::{Certificate, ClientConfig as RustlsConfig, ServerName};
    use std::time::SystemTime;

    struct SkipVerification;
    impl ServerCertVerifier for SkipVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &Certificate,
            _intermediates: &[Certificate],
            _server_name: &ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: SystemTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
    }

    let config = RustlsConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(SkipVerification))
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}
