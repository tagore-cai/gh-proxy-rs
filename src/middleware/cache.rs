use axum::{
    extract::State,
    http::{Request, Response},
    middleware::Next,
    body::Body,
};
use tracing::info;

use crate::{
    error::AppError,
    models::AppCache,
};

// Cache middleware
pub async fn cache_middleware(
    State(cache): State<AppCache>,
    request: Request<Body>,
    next: Next,
) -> Result<Response<Body>, AppError> {
    // Only cache GET requests
    if request.method() != "GET" {
        return Ok(next.run(request).await);
    }

    let path = get_request_path(&request);
    
    // Skip cache for redirect requests (q= prefix)
    if path.starts_with("q=") {
        return Ok(next.run(request).await);
    }

    // Try to get from cache first
    if cache.enabled {
        if let Some(cached_data) = cache.get(&path) {
            info!("Cache hit for: {}", path);
            let response = Response::builder()
                .status(axum::http::StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "application/octet-stream")
                .body(Body::from(cached_data))
                .map_err(|e| AppError::CacheError(e.to_string()))?;
            return Ok(response);
        } else {
            info!("Cache miss for: {}", path);
        }
    } else {
        info!("Cache disabled, proceeding to handler for: {}", path);
    }

    // Execute the next middleware/handler
    let response = next.run(request).await;

    // Cache the response if it's successful and caching is enabled
    let (parts, body) = response.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| AppError::CacheError(e.to_string()))?;
    
    // Only cache successful responses
    if cache.enabled && parts.status.is_success() {
        let size_before = cache.get_memory_usage();
        if cache.set(path.clone(), bytes.to_vec()) {
            let size_after = cache.get_memory_usage();
            info!("Caching response for: {}, size: {} bytes, total usage: {} bytes", 
                  path, bytes.len(), size_after);
            if size_after > size_before {
                info!("Cache memory usage increased from {} to {}", size_before, size_after);
            }
        } else {
            info!("Failed to cache response for: {} (likely due to size or memory limits)", path);
        }
    } else if cache.enabled {
        info!("Not caching response for: {} (status: {})", path, parts.status);
    }

    Ok(Response::from_parts(parts, Body::from(bytes)))
}

// Helper function to extract request path
fn get_request_path(request: &Request<Body>) -> String {
    let path_query = request
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(request.uri().path());

    let path = if path_query.starts_with("/") {
        path_query[1..].into()
    } else {
        path_query
    };
    path.into()
}