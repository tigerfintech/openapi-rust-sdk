//! Integration tests for QuoteClient — require real API credentials.
//! Run with: TIGER_RUN_INTEG=true cargo test --test integ_quote -- --nocapture

mod integ_support;

#[cfg(test)]
mod tests {
    use super::integ_support;
    use tigeropen::model::quote_requests::BriefRequest;
    use tigeropen::quote::QuoteClient;

    #[tokio::test]
    async fn test_integ_get_market_state() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let result = client.get_market_state("US").await;
        assert!(
            result.is_ok(),
            "get_market_state should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        assert!(!data.is_empty(), "market state list should not be empty");
    }

    #[tokio::test]
    async fn test_integ_get_real_time_quote() {
        if !integ_support::is_integ_run() {
            return;
        }
        let cfg = integ_support::integ_config();
        let client = QuoteClient::from_config(cfg);
        let req = BriefRequest {
            symbols: Some(vec!["AAPL".to_string()]),
            ..Default::default()
        };
        let result = client.get_real_time_quote(req).await;
        assert!(
            result.is_ok(),
            "get_real_time_quote should succeed: {:?}",
            result
        );
        let data = result.unwrap();
        assert!(
            !data.is_empty(),
            "brief result should not be empty for AAPL"
        );
    }
}
