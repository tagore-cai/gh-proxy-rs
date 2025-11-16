use axum::{
    body::Body,
    http::{header, Uri},
    response::Response,
};
use reqwest;
use sync_wrapper::SyncStream;
use futures::StreamExt;
use bytes::Bytes;
use tracing::info;
use crate::error::{AppError, Result};

// Helper function to collect bytes from stream
pub async fn collect_bytes(
    mut stream: impl futures::Stream<Item = std::result::Result<Bytes, reqwest::Error>> + Unpin,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(AppError::ReqwestError)?;
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

// Handle proxy requests
pub async fn handle_proxy(
    mut req: axum::extract::Request,
    client: &reqwest::Client,
    path_query: String,
) -> Result<Response> {
    // Remove HOST header
    req.headers_mut().remove(header::HOST);

    // Create new URI (clone path_query to avoid move)
    let new_uri = Uri::try_from(path_query.clone())
        .map_err(|_| AppError::InvalidRequest("Invalid URI".to_string()))?;

    *req.uri_mut() = new_uri;

    // Convert axum request to reqwest request
    let axum_request = req.map(|body| reqwest::Body::wrap_stream(SyncStream::new(body.into_data_stream())));
    let reqwest_request = reqwest::Request::try_from(axum_request)
        .map_err(|_| AppError::InvalidRequest("Failed to convert request".to_string()))?;

    // Execute request
    info!("Making HTTP request to: {}", path_query);
    let response = client.execute(reqwest_request).await
        .map_err(AppError::ReqwestError)?;

    // Get the response bytes and headers before consuming the response
    let headers = response.headers().clone();
    let status = response.status();
    
    info!("Received upstream response, status: {}", status);
    let response_bytes = collect_bytes(response.bytes_stream()).await?;

    // Build response
    let mut builder = Response::builder().status(status);
    if let Some(builder_headers) = builder.headers_mut() {
        *builder_headers = headers;
    }

    let response = builder.body(Body::from(response_bytes))
        .map_err(|e| AppError::CacheError(e.to_string()))?;
        
    Ok(response)
}