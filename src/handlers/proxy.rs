use axum::{
    body::Body,
    extract::{Request, State},
    http::header,
    response::{IntoResponse, Response},
};
use tracing::{info, warn, debug};

use crate::{
    config::Config,
    error::AppError,
    handlers,
    services,
    utils,
};

// Extract git URL from request
fn get_git_url(req: &Request<Body>) -> String {
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(req.uri().path());

    let path = if path_query.starts_with("/") {
        path_query[1..].into()
    } else {
        path_query
    };
    path.into()
}

// Main request handler
pub async fn handler(
    State((client, config)): State<(reqwest::Client, Config)>,
    req: Request<Body>,
) -> Result<Response, AppError> {
    use axum::http::Method;
    
    // Log incoming request
    let method = req.method().clone();
    let uri = req.uri().to_string();
    info!("Incoming request: {} {}", method, uri);

    // Handle OPTIONS requests for CORS
    if req.method() == Method::OPTIONS
        && req
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS)
    {
        debug!("Handling CORS preflight request");
        return handlers::handle_204();
    }

    let path = get_git_url(&req);

    // Handle redirects
    if path.starts_with("q=") {
        info!("Handling redirect for query: {}", &path[2..]);
        return handlers::handle_redirect(path.replace("q=", ""));
    }

    if utils::is_supported_url(&path, &config) {
        info!("Processing supported URL: {}", path);
        
        // Handle GitHub blob/raw URLs with jsDelivr if enabled
        if utils::GITHUB_BLOB_RAW.is_match(&path) && config.jsdelivr.enabled {
            info!("Redirecting to jsDelivr for GitHub blob URL");
            let new_url = path.replacen("/blob/", "@", 1).replacen(
                "github.com",
                "https://gcore.jsdelivr.net/gh",
                1,
            );
            return handlers::handle_redirect(new_url);
        }

        // Process the URL appropriately based on the service
        let final_path = utils::process_url(path, &config);
        info!("Proxying request to: {}", final_path);

        return services::handle_proxy(req, &client, final_path).await;
    }

    warn!("Unsupported URL requested: {}", path);
    // Default response for unsupported paths
    Ok("Proxy response placeholder".into_response())
}