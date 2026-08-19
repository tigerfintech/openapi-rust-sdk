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
pub mod tick_util;
pub mod varint;

pub use callbacks::*;
pub use push_client::*;
pub use push_message::*;
pub use tick_util::{convert_trade_tick, get_part_name_by_code, get_part_short_name_by_code, get_trade_cond_by_code, is_us_stock_symbol, PushTick, PushTradeTick};

#[cfg(test)]
mod tests;
