//! Integration tests for TradeClient — require real API credentials.
//! Only read-only operations (query orders, positions) — no order placement.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_trade -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::trade_requests::{OrdersRequest, PositionsRequest};
    use tigeropen::trade::TradeClient;

    #[tokio::test]
    async fn test_integ_get_positions() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = PositionsRequest::default();
        let result = client.get_positions(req).await;
        assert!(
            result.is_ok(),
            "get_positions should succeed: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_integ_get_orders() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = OrdersRequest::default();
        let result = client.get_orders(req).await;
        assert!(result.is_ok(), "get_orders should succeed: {:?}", result);
    }
}
