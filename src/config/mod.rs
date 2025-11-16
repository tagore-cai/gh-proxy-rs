use serde::Deserialize;
use std::net::SocketAddr;
use crate::error::{AppError, Result};

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_server")]
    pub server: ServerConfig,
    #[serde(default)]
    pub jsdelivr: JsDelivrConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub git_services: GitServicesConfig,
}

#[derive(Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_address")]
    pub address: SocketAddr,
}

#[derive(Clone, Deserialize, Default)]
pub struct JsDelivrConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Clone, Deserialize, Default)]
pub struct CacheConfig {
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    #[serde(default = "default_cache_max_capacity")]
    pub max_capacity: usize,
    #[serde(default = "default_cache_time_to_live")]
    pub time_to_live: u64,
    #[serde(default = "default_cache_max_memory")]
    pub max_memory: usize,
}

#[derive(Clone, Deserialize, Default)]
pub struct RateLimitConfig {
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,
    #[serde(default = "default_rate_limit_requests_per_minute")]
    pub requests_per_minute: u32,
}

#[derive(Clone, Deserialize, Default)]
pub struct GitServicesConfig {
    #[serde(default)]
    pub gitlab_enabled: bool,
    #[serde(default)]
    pub bitbucket_enabled: bool,
}

// Default values
fn default_address() -> SocketAddr {
    "127.0.0.1:4000".parse().expect("Invalid default address")
}

fn default_server() -> ServerConfig {
    ServerConfig {
        address: default_address(),
    }
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_max_capacity() -> usize {
    1000
}

fn default_cache_time_to_live() -> u64 {
    3600
}

fn default_cache_max_memory() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_rate_limit_enabled() -> bool {
    true
}

fn default_rate_limit_requests_per_minute() -> u32 {
    60
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: default_server(),
            jsdelivr: JsDelivrConfig { enabled: false },
            cache: CacheConfig {
                enabled: default_cache_enabled(),
                max_capacity: default_cache_max_capacity(),
                time_to_live: default_cache_time_to_live(),
                max_memory: default_cache_max_memory(),
            },
            rate_limit: RateLimitConfig {
                enabled: default_rate_limit_enabled(),
                requests_per_minute: default_rate_limit_requests_per_minute(),
            },
            git_services: GitServicesConfig {
                gitlab_enabled: false,
                bitbucket_enabled: false,
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let mut cfg = config::Config::builder()
            .add_source(config::File::with_name(path).required(false))
            .add_source(config::Environment::with_prefix("GH_PROXY"));

        // Set defaults
        cfg = cfg.set_default("server.address", "127.0.0.1:4000")
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("jsdelivr.enabled", false)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("cache.enabled", true)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("cache.max_capacity", 1000)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("cache.time_to_live", 3600)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("cache.max_memory", 104857600) // 100MB
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("rate_limit.enabled", true)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("rate_limit.requests_per_minute", 60)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("git_services.gitlab_enabled", false)
            .map_err(AppError::ConfigError)?;
        cfg = cfg.set_default("git_services.bitbucket_enabled", false)
            .map_err(AppError::ConfigError)?;

        let config = cfg.build().map_err(AppError::ConfigError)?;
        config.try_deserialize().map_err(AppError::ConfigError)
    }
}