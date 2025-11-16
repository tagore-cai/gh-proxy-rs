use crate::config::Config;
use axum::http::HeaderMap;

mod regex;
mod url;
pub use regex::*;
pub use url::*;

// Process URL based on the service
pub fn process_url(path: String, config: &Config) -> String {
    // Handle GitHub blob URLs
    if regex::GITHUB_BLOB_RAW.is_match(&path) {
        return path.replacen("/blob/", "/raw/", 1);
    }
    
    // Handle GitLab blob URLs
    if config.git_services.gitlab_enabled && regex::GITLAB_BLOBS.is_match(&path) {
        // Convert GitLab blob URL to raw URL
        return path.replace("/-/blob/", "/-/raw/").replace("/blob/", "/raw/");
    }
    
    // For all other URLs, return as is
    path
}

// Get client IP from request headers
pub fn get_client_ip(headers: &HeaderMap) -> &str {
    headers
        .get("x-forwarded-for")
        .and_then(|hv| hv.to_str().ok())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|hv| hv.to_str().ok())
        })
        .unwrap_or("unknown")
}