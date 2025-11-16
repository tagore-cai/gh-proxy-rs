use axum::{
    extract::State,
    http::{HeaderMap, Request},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::error::AppError;

// Rate limit entry
#[derive(Clone)]
pub struct RateLimitEntry {
    pub count: u32,
    pub timestamp: u64,
}

// Rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    pub limits: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    pub enabled: bool,
    pub requests_per_minute: u32,
}

impl RateLimiter {
    pub fn new(enabled: bool, requests_per_minute: u32) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            enabled,
            requests_per_minute,
        }
    }

    pub fn is_allowed(&self, key: &str) -> bool {
        if !self.enabled {
            return true;
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        let mut limits = match self.limits.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        
        let entry = limits.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            timestamp: current_time,
        });

        // Reset counter if more than a minute has passed
        if current_time - entry.timestamp >= 60 {
            info!("Resetting rate limit counter for: {}", key);
            entry.count = 1;
            entry.timestamp = current_time;
            return true;
        }

        // Check if limit is exceeded
        if entry.count >= self.requests_per_minute {
            warn!("Rate limit exceeded for: {}", key);
            return false;
        }

        // Increment count
        entry.count += 1;
        info!("Incrementing rate limit counter for: {} (current: {}/{})", key, entry.count, self.requests_per_minute);
        true
    }
}

// Get client IP from request headers
pub fn get_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|hv| hv.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string()
}

// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !rate_limiter.enabled {
        info!("Rate limiting disabled, proceeding with request");
        return Ok(next.run(request).await);
    }

    let client_ip = get_client_ip(request.headers());
    
    if !rate_limiter.is_allowed(&client_ip) {
        warn!("Request from {} blocked due to rate limit", client_ip);
        return Err(AppError::RateLimitError("Rate limit exceeded".to_string()));
    }

    info!("Request from {} passed rate limit check", client_ip);
    Ok(next.run(request).await)
}