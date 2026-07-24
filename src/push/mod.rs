//! Push module - TCP+TLS push client
//!
//! Receives real-time market data and account push notifications via
//! a raw TCP+TLS persistent connection.
//! Supports connection authentication, subscribe/unsubscribe, callbacks,
//! heartbeat keep-alive, and automatic reconnection.

mod callbacks;
pub mod pb;
pub mod proto_message;
mod push_client;
mod push_message;
pub mod varint;

pub use callbacks::*;
pub use push_client::*;
pub use push_message::*;

#[cfg(test)]
mod tests;
