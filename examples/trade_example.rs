//! Trade example - covers every TradeClient public method with typed responses.
//!
//! Config is auto-discovered from:
//!   1. ./tiger_openapi_config.properties
//!   2. ~/.tigeropen/tiger_openapi_config.properties
//!   TIGER_CONFIG_PATH env var overrides both.
//!
//! Executes a real low-price limit order: BUY 1 AAPL @ $1.00 (will not fill),
//! immediately modifies the price, then cancels.
//!
//! Individual failures do not abort subsequent calls; a final PASS/FAIL/SKIP summary is printed.
//!
//! Run: `TIGER_CONFIG_PATH=~/.tigeropen/tiger_openapi_config.properties cargo run --example trade_example`

use std::time::{SystemTime, UNIX_EPOCH};

use tigeropen::client::http_client::HttpClient;
use tigeropen::config::ClientConfig;
use tigeropen::model::order::limit_order;
use tigeropen::model::trade_requests::{
    AggregateAssetsRequest, AnalyticsAssetRequest, AssetsRequest, EstimateTradableQuantityRequest,
    GetOrderRequest, ManagedAccountsRequest, OptionExerciseCheckRequest,
    OptionExercisePositionRequest, OptionExerciseRecordsRequest, OrderTransactionsRequest,
    OrdersRequest, PositionTransferExternalRecordsRequest, PositionTransferRecordsRequest,
    PositionsRequest, SegmentFundRequest,
};
use tigeropen::trade::TradeClient;

enum Outcome {
    Pass,
    Fail,
    Skip,
}

struct RunResult {
    name: String,
    outcome: Outcome,
    detail: String,
}

fn ok(results: &mut Vec<RunResult>, name: &str, note: impl Into<String>) {
    let note = note.into();
    println!("[ OK ] {:<50} {}", name, note);
    results.push(RunResult { name: name.into(), outcome: Outcome::Pass, detail: note });
}

fn fail(results: &mut Vec<RunResult>, name: &str, err: impl std::fmt::Display) {
    let detail = format!("{}", err);
    println!("[FAIL] {:<50} {}", name, detail);
    results.push(RunResult { name: name.into(), outcome: Outcome::Fail, detail });
}

fn skip(results: &mut Vec<RunResult>, name: &str, reason: impl Into<String>) {
    let reason = reason.into();
    println!("[SKIP] {:<50} {}", name, reason);
    results.push(RunResult { name: name.into(), outcome: Outcome::Skip, detail: reason });
}

fn print_summary(results: &[RunResult]) {
    let (mut p, mut f, mut s) = (0, 0, 0);
    for r in results {
        match r.outcome {
            Outcome::Pass => p += 1,
            Outcome::Fail => f += 1,
            Outcome::Skip => s += 1,
        }
    }
    println!("\n================ SUMMARY ================");
    println!("PASS={}  FAIL={}  SKIP={}  TOTAL={}", p, f, s, results.len());
    if f > 0 {
        println!("\nFailures:");
        for r in results {
            if matches!(r.outcome, Outcome::Fail) {
                println!("  - {}: {}", r.name, r.detail);
            }
        }
    }
    println!("=========================================");
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Support TIGER_CONFIG_PATH env var to avoid copying config files into the repo
    let config = match std::env::var("TIGER_CONFIG_PATH") {
        Ok(path) => ClientConfig::builder().properties_file(&path).build()?,
        Err(_) => ClientConfig::builder().build()?,
    };
    println!("tiger_id={} account={}\n", config.tiger_id, config.account);

    let account = config.account.clone();
    let http = HttpClient::new(config.clone());
    let tc = match &config.secret_key {
        Some(sk) => TradeClient::with_secret_key(&http, &account, sk),
        None => TradeClient::new(&http, &account),
    };

    let mut results: Vec<RunResult> = Vec::new();

    println!("=== Contract 查询 ===");
    match tc.get_contract("AAPL", "STK").await {
        Ok(cs) if !cs.is_empty() => ok(
            &mut results,
            "Contract(AAPL, STK)",
            format!(
                "{} contractId={:?} exchange={:?}",
                cs[0].symbol, cs[0].contract_id, cs[0].exchange
            ),
        ),
        Ok(_) => ok(&mut results, "Contract(AAPL, STK)", "(empty)"),
        Err(e) => fail(&mut results, "Contract(AAPL, STK)", e),
    }

    match tc.get_contracts(&["AAPL", "TSLA"], "STK").await {
        Ok(cs) => {
            let names: Vec<String> = cs.iter().map(|c| c.symbol.clone()).collect();
            ok(
                &mut results,
                "Contracts([AAPL TSLA])",
                format!("count={} {}", cs.len(), names.join(",")),
            );
        }
        Err(e) => fail(&mut results, "Contracts([AAPL TSLA])", e),
    }

    match tc.get_quote_contract("AAPL", "OPT", "20260619").await {
        Ok(cs) => ok(
            &mut results,
            "QuoteContract(AAPL OPT)",
            format!("count={}", cs.len()),
        ),
        Err(e) => fail(&mut results, "QuoteContract(AAPL OPT)", e),
    }

    println!("\n=== 账户/持仓 查询 ===");
    match tc.get_assets(AssetsRequest::default()).await {
        Ok(assets) if !assets.is_empty() => {
            let a = &assets[0];
            ok(
                &mut results,
                "Assets",
                format!(
                    "account={} buyingPower={:.2} netLiquidation={:.2} segments={}",
                    a.account,
                    a.buying_power,
                    a.net_liquidation,
                    a.segments.len()
                ),
            );
        }
        Ok(_) => ok(&mut results, "Assets", "(empty)"),
        Err(e) => fail(&mut results, "Assets", e),
    }

    match tc.get_prime_assets(AssetsRequest::default()).await {
        Ok(Some(pa)) => {
            let total_bp: f64 = pa.segments.iter().map(|s| s.buying_power).sum();
            ok(
                &mut results,
                "PrimeAssets",
                format!(
                    "account={} segments={} totalBuyingPower={:.2}",
                    pa.account_id,
                    pa.segments.len(),
                    total_bp
                ),
            );
        }
        Ok(None) => ok(&mut results, "PrimeAssets", "(empty)"),
        Err(e) => fail(&mut results, "PrimeAssets", e),
    }

    match tc.get_positions(PositionsRequest::default()).await {
        Ok(ps) => {
            let total_mv: f64 = ps
                .iter()
                .map(|p| p.market_value.unwrap_or(0.0))
                .sum();
            ok(
                &mut results,
                "Positions",
                format!("count={} totalMarketValue={:.2}", ps.len(), total_mv),
            );
        }
        Err(e) => fail(&mut results, "Positions", e),
    }

    println!("\n=== 订单 查询 ===");
    match tc.get_orders(OrdersRequest::default()).await {
        Ok(os) => ok(&mut results, "Orders", format!("count={}", os.len())),
        Err(e) => fail(&mut results, "Orders", e),
    }
    match tc.get_active_orders(OrdersRequest::default()).await {
        Ok(os) => ok(&mut results, "ActiveOrders", format!("count={}", os.len())),
        Err(e) => fail(&mut results, "ActiveOrders", e),
    }
    match tc.get_inactive_orders(OrdersRequest::default()).await {
        Ok(os) => ok(&mut results, "InactiveOrders", format!("count={}", os.len())),
        Err(e) => fail(&mut results, "InactiveOrders", e),
    }
    let now = now_ms();
    match tc
        .get_filled_orders(OrdersRequest {
            start_date: Some(now - 30 * 24 * 3600 * 1000),
            end_date: Some(now),
            ..Default::default()
        })
        .await
    {
        Ok(os) => ok(
            &mut results,
            "FilledOrders",
            format!("count={} (last 30d)", os.len()),
        ),
        Err(e) => fail(&mut results, "FilledOrders", e),
    }

    // Fetch an existing order id for subsequent tests
    let mut existing_order_id: i64 = 0;
    if let Ok(orders) = tc.get_orders(OrdersRequest::default()).await {
        if !orders.is_empty() {
            existing_order_id = orders[0].id;
        }
    }

    if existing_order_id != 0 {
        let name = format!("GetOrder({})", existing_order_id);
        match tc
            .get_order(GetOrderRequest {
                id: Some(existing_order_id),
                ..Default::default()
            })
            .await
        {
            Ok(Some(o)) => ok(&mut results, &name, format!("symbol={} status={:?}", o.symbol, o.status)),
            Ok(None) => ok(&mut results, &name, "(empty)"),
            Err(e) => fail(&mut results, &name, e),
        }

        let name = format!("OrderTransactions({})", existing_order_id);
        match tc
            .get_order_transactions(OrderTransactionsRequest {
                order_id: Some(existing_order_id),
                symbol: Some("AAPL".to_string()),
                sec_type: Some("STK".to_string()),
                ..Default::default()
            })
            .await
        {
            Ok(txs) => ok(&mut results, &name, format!("count={}", txs.len())),
            Err(e) => fail(&mut results, &name, e),
        }
    } else {
        skip(&mut results, "GetOrder", "no existing order");
        skip(&mut results, "OrderTransactions", "no existing order");
    }

    println!("\n=== 下单/改单/撤单 ===");
    let mut order_req = limit_order(&account, "AAPL", "STK", "BUY", 1, 1.00);
    order_req.market = Some("US".into());
    order_req.currency = Some("USD".into());
    order_req.time_in_force = Some("DAY".into());

    match tc.preview_order(order_req.clone()).await {
        Ok(Some(p)) => ok(
            &mut results,
            "PreviewOrder",
            format!(
                "isPass={} commission={:.2} initMargin={:.2}",
                p.is_pass, p.commission, p.init_margin
            ),
        ),
        Ok(None) => ok(&mut results, "PreviewOrder", "(empty)"),
        Err(e) => fail(&mut results, "PreviewOrder", e),
    }

    match tc.place_order(order_req.clone()).await {
        Ok(Some(placed)) => {
            ok(
                &mut results,
                "PlaceOrder",
                format!("id={} orderId={}", placed.id, placed.order_id),
            );

            let mut modify_req = order_req.clone();
            modify_req.limit_price = Some(1.50);
            let name = format!("ModifyOrder({})", placed.id);
            match tc.modify_order(placed.id, modify_req).await {
                Ok(Some(m)) => ok(&mut results, &name, format!("id={}", m.id)),
                Ok(None) => ok(&mut results, &name, "(empty)"),
                Err(e) => fail(&mut results, &name, e),
            }

            let name = format!("CancelOrder({})", placed.id);
            match tc.cancel_order(placed.id).await {
                Ok(Some(c)) => ok(&mut results, &name, format!("id={}", c.id)),
                Ok(None) => ok(&mut results, &name, "(empty)"),
                Err(e) => fail(&mut results, &name, e),
            }
        }
        Ok(None) => {
            fail(&mut results, "PlaceOrder", "empty response");
            skip(&mut results, "ModifyOrder", "PlaceOrder returned empty");
            skip(&mut results, "CancelOrder", "PlaceOrder returned empty");
        }
        Err(e) => {
            fail(&mut results, "PlaceOrder", e);
            skip(&mut results, "ModifyOrder", "PlaceOrder failed");
            skip(&mut results, "CancelOrder", "PlaceOrder failed");
        }
    }

    println!("\n=== v0.4.0 新增 smoke tests ===");

    // ManagedAccounts
    match tc.get_managed_accounts(ManagedAccountsRequest::default()).await {
        Ok(accs) => ok(&mut results, "ManagedAccounts", format!("count={}", accs.len())),
        Err(e) => fail(&mut results, "ManagedAccounts", e),
    }

    // AnalyticsAsset (seg_type=SEC, date range last week)
    match tc
        .get_analytics_asset(AnalyticsAssetRequest {
            seg_type: Some("SEC".to_string()),
            start_date: Some("2025-05-01".to_string()),
            end_date: Some("2025-05-07".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(&mut results, "AnalyticsAsset", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "AnalyticsAsset", e),
    }

    // AggregateAssets
    match tc.get_aggregate_assets(AggregateAssetsRequest::default()).await {
        Ok(Some(a)) => ok(
            &mut results,
            "AggregateAssets",
            format!("netLiquidation={:.2}", a.net_liquidation),
        ),
        Ok(None) => ok(&mut results, "AggregateAssets", "(empty)"),
        Err(e) => fail(&mut results, "AggregateAssets", e),
    }

    // EstimateTradableQuantity
    match tc
        .get_estimate_tradable_quantity(EstimateTradableQuantityRequest {
            symbol: Some("AAPL".to_string()),
            sec_type: Some("STK".to_string()),
            action: Some("BUY".to_string()),
            order_type: Some("LMT".to_string()),
            limit_price: Some(200.0),
            ..Default::default()
        })
        .await
    {
        Ok(Some(q)) => ok(
            &mut results,
            "EstimateTradableQuantity",
            format!("tradable={} cashBuy={}", q.tradable_quantity, q.max_cash_buy_quantity),
        ),
        Ok(None) => ok(&mut results, "EstimateTradableQuantity", "(empty)"),
        Err(e) => fail(&mut results, "EstimateTradableQuantity", e),
    }

    // SegmentFundAvailable
    match tc
        .get_segment_fund_available(SegmentFundRequest {
            from_segment: Some("SEC".to_string()),
            to_segment: Some("FUT".to_string()),
            currency: Some("USD".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(funds) => ok(&mut results, "SegmentFundAvailable", format!("count={}", funds.len())),
        Err(e) => fail(&mut results, "SegmentFundAvailable", e),
    }

    // SegmentFundHistory
    match tc
        .get_segment_fund_history(SegmentFundRequest {
            limit: Some(10),
            ..Default::default()
        })
        .await
    {
        Ok(items) => ok(&mut results, "SegmentFundHistory", format!("count={}", items.len())),
        Err(e) => fail(&mut results, "SegmentFundHistory", e),
    }

    // PositionTransferRecords (7-day window to avoid max_date.limit)
    match tc
        .get_position_transfer_records(PositionTransferRecordsRequest {
            since_date: Some("2026-06-01".to_string()),
            to_date: Some("2026-06-09".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(records) => ok(
            &mut results,
            "PositionTransferRecords",
            format!("count={}", records.len()),
        ),
        Err(e) => fail(&mut results, "PositionTransferRecords", e),
    }

    // PositionTransferExternalRecords (7-day window to avoid max_date.limit)
    match tc
        .get_position_transfer_external_records(PositionTransferExternalRecordsRequest {
            since_date: Some("2026-06-01".to_string()),
            to_date: Some("2026-06-09".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(records) => ok(
            &mut results,
            "PositionTransferExternalRecords",
            format!("count={}", records.len()),
        ),
        Err(e) => fail(&mut results, "PositionTransferExternalRecords", e),
    }

    // =========================================================================
    // v0.4.2: 期权行权
    // =========================================================================
    // GetOptionExercisePositions (Exercise)
    match tc
        .get_option_exercise_positions(OptionExercisePositionRequest {
            exercise_type: Some("Exercise".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(Some(r)) => ok(
            &mut results,
            "GetOptionExercisePositions(Exercise)",
            format!("rows={} pageCount={}", r.items.len(), r.page_count),
        ),
        Ok(None) => ok(&mut results, "GetOptionExercisePositions(Exercise)", "(empty)"),
        Err(e) => fail(&mut results, "GetOptionExercisePositions(Exercise)", e),
    }

    // GetOptionExercisePositions (Expire)
    let exercise_positions = match tc
        .get_option_exercise_positions(OptionExercisePositionRequest {
            exercise_type: Some("Expire".to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(Some(r)) => {
            ok(
                &mut results,
                "GetOptionExercisePositions(Expire)",
                format!("rows={} pageCount={}", r.items.len(), r.page_count),
            );
            r.items
        }
        Ok(None) => {
            ok(&mut results, "GetOptionExercisePositions(Expire)", "(empty)");
            vec![]
        }
        Err(e) => {
            fail(&mut results, "GetOptionExercisePositions(Expire)", e);
            vec![]
        }
    };

    // GetOptionExerciseRecords
    match tc
        .get_option_exercise_records(OptionExerciseRecordsRequest {
            page: Some(1),
            size: Some(10),
            ..Default::default()
        })
        .await
    {
        Ok(Some(r)) => ok(
            &mut results,
            "GetOptionExerciseRecords",
            format!("rows={} itemCount={}", r.items.len(), r.item_count),
        ),
        Ok(None) => ok(&mut results, "GetOptionExerciseRecords", "(empty)"),
        Err(e) => fail(&mut results, "GetOptionExerciseRecords", e),
    }

    // CheckOptionExercise (conditional — needs an exercisable position)
    if let Some(p) = exercise_positions.first() {
        match tc
            .option_exercise_check(OptionExerciseCheckRequest {
                contract_id: Some(p.contract_id),
                exercise_type: Some("Expire".to_string()),
                quantity: Some(p.available_quantity),
                ..Default::default()
            })
            .await
        {
            Ok(Some(r)) => ok(
                &mut results,
                "CheckOptionExercise",
                format!("symbol={} availableQty={}", r.symbol, r.available_quantity),
            ),
            Ok(None) => ok(&mut results, "CheckOptionExercise", "(empty)"),
            Err(e) => fail(&mut results, "CheckOptionExercise", e),
        }
    } else {
        skip(&mut results, "CheckOptionExercise", "no exercisable positions");
    }

    print_summary(&results);
    Ok(())
}
