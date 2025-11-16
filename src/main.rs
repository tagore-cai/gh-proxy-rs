use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import modules
mod config;
mod error;
mod models;
mod handlers;
mod services;
mod utils;
mod middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gh_proxy_rs=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting gh-proxy-rs server");

    // Load configuration
    let config = config::Config::from_file("config.toml")?;
    tracing::info!("Configuration loaded: server={} cache={} rate_limit={} requests_per_minute={}", 
                   config.server.address, 
                   config.cache.enabled, 
                   config.rate_limit.enabled, 
                   config.rate_limit.requests_per_minute);
    
    let cache = models::AppCache::with_memory_limit(
        config.cache.enabled,
        config.cache.max_capacity,
        config.cache.max_memory,
        config.cache.time_to_live,
    );
    
    let rate_limiter = middleware::RateLimiter::new(
        config.rate_limit.enabled,
        config.rate_limit.requests_per_minute,
    );

    // Create HTTP client
    let client = reqwest::Client::new();

    // Create app with state
    let app = Router::new()
        .fallback(handlers::handler)
        .route_layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            middleware::rate_limit_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            cache.clone(),
            middleware::cache_middleware,
        ))
        .with_state((client, config.clone()));

    // Bind and serve
    let listener = tokio::net::TcpListener::bind(config.server.address).await?;
    tracing::info!("Listening on {}", config.server.address);
    axum::serve(listener, app).await?;
    
    Ok(())
}