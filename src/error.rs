use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] http::Error),
    
    #[error("URI error: {0}")]
    UriError(#[from] http::uri::InvalidUri),
    
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
    
    #[error("Address parse error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Rate limit error: {0}")]
    RateLimitError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConfigError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            AppError::HttpError(_) => (StatusCode::BAD_REQUEST, "HTTP error"),
            AppError::UriError(_) => (StatusCode::BAD_REQUEST, "Invalid URI"),
            AppError::ReqwestError(_) => (StatusCode::SERVICE_UNAVAILABLE, "Service unavailable"),
            AppError::RegexError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Regex error"),
            AppError::AddrParseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Address parse error"),
            AppError::CacheError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error"),
            AppError::RateLimitError(_) => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            AppError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "Invalid request"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string()
        }));

        (status, body).into_response()
    }
}

// 为Result类型创建别名
pub type Result<T> = std::result::Result<T, AppError>;