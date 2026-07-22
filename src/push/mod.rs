//! Push module - TCP+TLS push client
//!
//! Receives real-time market data and account push notifications via
//! a raw TCP+TLS persistent connection.
//! Supports connection authentication, subscribe/unsubscribe, callbacks,
//! heartbeat keep-alive, and automatic reconnection.

pub mod pb;
pub mod varint;
pub mod proto_message;
mod push_message;
mod callbacks;
mod push_client;

pub use push_message::*;
pub use callbacks::*;
pub use push_client::*;

#[cfg(test)]
mod tests;
