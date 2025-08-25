use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

use crate::ctx::{Context, event_config::EventConfig};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable {0} is not set")]
    MissingEnvVar(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Invalid port: {0}")]
    InvalidPort(u16),
    #[error("Invalid event configuration: {0}")]
    InvalidEventConfig(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CorsConfig {
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,
    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,
    #[serde(default = "default_max_age")]
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: default_allowed_origins(),
            allowed_methods: default_allowed_methods(),
            allowed_headers: default_allowed_headers(),
            max_age: default_max_age(),
        }
    }
}

fn default_allowed_origins() -> Vec<String> {
    // 默认允许所有源，相当于没有CORS限制
    vec![]
}

fn default_allowed_methods() -> Vec<String> {
    // 默认允许所有方法，相当于没有CORS限制
    vec![]
}

fn default_allowed_headers() -> Vec<String> {
    // 默认允许所有头部，相当于没有CORS限制
    vec![]
}

fn default_max_age() -> u64 {
    3600
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit: default_rate_limit(),
            max_body_size: default_max_body_size(),
        }
    }
}

fn default_rate_limit() -> u32 {
    100
}



fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub host_url: String,
    pub grpc_url: String,
    pub database_url: String,
    #[serde(default)]
    pub cors: CorsConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default)]
    pub events: EventConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host_url: "127.0.0.1:3000".to_string(),
            grpc_url: "grpc://8.210.45.192:16610".to_string(),
            database_url: "postgres://postgres:postgres@127.0.0.1/postgres".to_string(),
            cors: CorsConfig::default(),
            security: SecurityConfig::default(),
            log_level: "info".to_string(),
            environment: "development".to_string(),
            events: EventConfig::default(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // Load config from environment variables
        if let Ok(host_url) = env::var("TONDI_SCAN_HOST_URL") {
            config.host_url = host_url;
        }
        
        if let Ok(grpc_url) = env::var("TONDI_SCAN_GRPC_URL") {
            config.grpc_url = grpc_url;
        }
        
        if let Ok(database_url) = env::var("TONDI_SCAN_DATABASE_URL") {
            config.database_url = database_url;
        }
        
        if let Ok(log_level) = env::var("TONDI_SCAN_LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        if let Ok(environment) = env::var("TONDI_SCAN_ENVIRONMENT") {
            config.environment = environment;
        }
        
        // Load CORS configuration from environment variables
        if let Ok(allowed_origins) = env::var("TONDI_SCAN_CORS_ALLOWED_ORIGINS") {
            if allowed_origins == "*" || allowed_origins.is_empty() {
                // If set to "*" or empty, allow all origins
                config.cors.allowed_origins = vec![];
            } else {
                config.cors.allowed_origins = allowed_origins
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        if let Ok(allowed_methods) = env::var("TONDI_SCAN_CORS_ALLOWED_METHODS") {
            if allowed_methods == "*" || allowed_methods.is_empty() {
                // If set to "*" or empty, allow all methods
                config.cors.allowed_methods = vec![];
            } else {
                config.cors.allowed_methods = allowed_methods
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        if let Ok(allowed_headers) = env::var("TONDI_SCAN_CORS_ALLOWED_HEADERS") {
            if allowed_headers == "*" || allowed_headers.is_empty() {
                // If set to "*" or empty, allow all headers
                config.cors.allowed_headers = vec![];
            } else {
                config.cors.allowed_headers = allowed_headers
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        if let Ok(max_age) = env::var("TONDI_SCAN_CORS_MAX_AGE") {
            if let Ok(age) = max_age.parse() {
                config.cors.max_age = age;
            }
        }
        
        // Load security configuration from environment variables
        if let Ok(rate_limit) = env::var("TONDI_SCAN_RATE_LIMIT") {
            if let Ok(limit) = rate_limit.parse() {
                config.security.rate_limit = limit;
            }
        }
        
        if let Ok(max_body_size) = env::var("TONDI_SCAN_MAX_BODY_SIZE") {
            if let Ok(size) = max_body_size.parse() {
                config.security.max_body_size = size;
            }
        }
        
        // Load event configuration from environment variables
        if let Ok(enabled_events) = env::var("TONDI_SCAN_ENABLED_EVENTS") {
            config.events.enabled_events = enabled_events
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(event_strategy) = env::var("TONDI_SCAN_EVENT_STRATEGY") {
            config.events.event_strategy = match event_strategy.as_str() {
                "batch" => {
                    let batch_size = env::var("TONDI_SCAN_BATCH_SIZE")
                        .unwrap_or_else(|_| "100".to_string())
                        .parse()
                        .unwrap_or(100);
                    let batch_timeout_ms = env::var("TONDI_SCAN_BATCH_TIMEOUT_MS")
                        .unwrap_or_else(|_| "100".to_string())
                        .parse()
                        .unwrap_or(100);
                    EventStrategy::Batch { batch_size, batch_timeout_ms }
                }
                "priority" => {
                    let high_priority = env::var("TONDI_SCAN_HIGH_PRIORITY_EVENTS")
                        .unwrap_or_else(|_| "block-added,utxos-changed".to_string())
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let medium_priority = env::var("TONDI_SCAN_MEDIUM_PRIORITY_EVENTS")
                        .unwrap_or_else(|_| "virtual-chain-changed".to_string())
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    let low_priority = env::var("TONDI_SCAN_LOW_PRIORITY_EVENTS")
                        .unwrap_or_else(|_| "new-block-template".to_string())
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    EventStrategy::Priority { high_priority, medium_priority, low_priority }
                }
                _ => EventStrategy::RealTime,
            };
        }
        
        if let Ok(buffer_size) = env::var("TONDI_SCAN_BUFFER_SIZE") {
            if let Ok(size) = buffer_size.parse() {
                config.events.buffer_size = size;
            }
        }
        
        if let Ok(enable_deduplication) = env::var("TONDI_SCAN_ENABLE_DEDUPLICATION") {
            config.events.enable_deduplication = enable_deduplication.parse().unwrap_or(true);
        }
        
        // Validate config
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate port
        if let Some(port) = self.host_url.split(':').last() {
            if let Ok(port_num) = port.parse::<u16>() {
                if port_num == 0 {
                    return Err(ConfigError::InvalidPort(port_num));
                }
            }
        }
        
        // Validate database URL
        if !self.database_url.starts_with("postgres://") {
            return Err(ConfigError::InvalidUrl(self.database_url.clone()));
        }
        
        // Validate event configuration
        self.events.validate()
            .map_err(|e| ConfigError::InvalidEventConfig(e))?;
        
        Ok(())
    }
    
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
    
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
}

impl FromRef<Context> for &Config {
    fn from_ref<'a>(ctx: &'a Context) -> Self {
        let this = &*ctx.config;
        // Safety: 'static Context
        unsafe { std::mem::transmute::<&'a Config, Self>(this) }
    }
}
