//! 传输层模块：HttpClient、重试策略、API 请求/响应、错误分类。

pub mod api_request;
pub mod api_response;
pub mod decode;
pub mod errors;
pub mod http_client;
pub mod retry;
