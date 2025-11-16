use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
};

use crate::error::{AppError, Result};

mod proxy;
pub use proxy::handler;

// Handle 204 No Content response for CORS preflight requests
pub fn handle_204() -> Result<Response> {
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::NO_CONTENT;
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("GET,POST,PUT,PATCH,TRACE,DELETE,HEAD,OPTIONS"),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_MAX_AGE,
        header::HeaderValue::from_static("1728000"),
    );
    Ok(res)
}

// Handle redirect responses
pub fn handle_redirect(query_string: String) -> Result<Response> {
    let location = format!("/{}", query_string);
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::FOUND;
    let header_value = header::HeaderValue::from_str(&location)
        .map_err(|_| AppError::InvalidRequest("Invalid redirect location".to_string()))?;
    res.headers_mut().insert(header::LOCATION, header_value);
    Ok(res)
}