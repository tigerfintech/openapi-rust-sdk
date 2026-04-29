fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files: Vec<&str> = vec![
        "proto/SocketCommon.proto",
        "proto/Request.proto",
        "proto/Response.proto",
        "proto/PushData.proto",
        "proto/QuoteData.proto",
        "proto/QuoteBasicData.proto",
        "proto/QuoteBBOData.proto",
        "proto/QuoteDepthData.proto",
        "proto/TradeTickData.proto",
        "proto/TickData.proto",
        "proto/AssetData.proto",
        "proto/PositionData.proto",
        "proto/OrderStatusData.proto",
        "proto/OrderTransactionData.proto",
        "proto/StockTopData.proto",
        "proto/OptionTopData.proto",
        "proto/KlineData.proto",
    ];
    prost_build::compile_protos(&proto_files, &["proto/"])?;
    Ok(())
}
