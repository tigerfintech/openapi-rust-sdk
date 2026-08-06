//! Integration tests for TradeClient — require real API credentials.
//! Only read-only operations (query orders, positions, assets) — no order placement.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_trade -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::trade_requests::{AssetsRequest, OrdersRequest, PositionsRequest};
    use tigeropen::model::{Asset, Order, Position};
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
        let data: Vec<Position> = result.unwrap();
        // Positions may legitimately be empty (e.g. paper account with no holdings).
        // When entries exist, validate key fields.
        if !data.is_empty() {
            let p = &data[0];
            assert!(
                p.symbol.as_ref().is_some_and(|s| !s.is_empty()),
                "Position.symbol should be non-empty, got {:?}",
                p.symbol
            );
            assert!(
                p.position.is_some(),
                "Position.position (quantity) should be present, got None"
            );
            assert!(
                p.sec_type.is_some(),
                "Position.sec_type should be present, got None"
            );
            assert!(
                p.market.is_some(),
                "Position.market should be present, got None"
            );
            assert!(
                p.latest_price.map(|v| v >= 0.0).unwrap_or(true),
                "Position.latest_price should be >= 0, got {:?}",
                p.latest_price
            );
        }
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
        let data: Vec<Order> = result.unwrap();
        // Orders may legitimately be empty (paper account with no trades).
        // When entries exist, validate key fields.
        if !data.is_empty() {
            let o = &data[0];
            assert!(
                !o.account.is_empty(),
                "Order.account should be non-empty, got {:?}",
                o.account
            );
            assert!(
                !o.status.is_empty(),
                "Order.status should be non-empty, got {:?}",
                o.status
            );
            assert!(
                !o.action.is_empty(),
                "Order.action should be non-empty (BUY/SELL), got {:?}",
                o.action
            );
            assert!(
                !o.order_type.is_empty(),
                "Order.order_type should be non-empty, got {:?}",
                o.order_type
            );
            assert!(
                o.total_quantity >= 0,
                "Order.total_quantity should be >= 0, got {}",
                o.total_quantity
            );
            assert!(
                o.filled_quantity >= 0,
                "Order.filled_quantity should be >= 0, got {}",
                o.filled_quantity
            );
        }
    }

    #[tokio::test]
    async fn test_integ_get_assets() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = TradeClient::from_config(cfg);
        let req = AssetsRequest::default();
        let result = client.get_assets(req).await;
        assert!(result.is_ok(), "get_assets should succeed: {:?}", result);
        let data: Vec<Asset> = result.unwrap();
        // Assets should typically not be empty for a valid account.
        assert!(
            !data.is_empty(),
            "assets result should not be empty for a valid account"
        );
        let a = &data[0];
        assert!(
            !a.account.is_empty(),
            "Asset.account should be non-empty, got {:?}",
            a.account
        );
        assert!(
            !a.currency.is_empty(),
            "Asset.currency should be non-empty, got {:?}",
            a.currency
        );
        assert!(
            a.net_liquidation >= 0.0,
            "Asset.net_liquidation should be >= 0, got {}",
            a.net_liquidation
        );
        assert!(
            a.buying_power >= 0.0,
            "Asset.buying_power should be >= 0, got {}",
            a.buying_power
        );
    }
}
