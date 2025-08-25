use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

use crate::ctx::Context;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable {0} is not set")]
    MissingEnvVar(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Invalid port: {0}")]
    InvalidPort(u16),
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
    vec!["http://localhost:3000".to_string(), "http://localhost:8080".to_string()]
}

fn default_allowed_methods() -> Vec<String> {
    vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()]
}

fn default_allowed_headers() -> Vec<String> {
    vec!["Content-Type".to_string(), "Authorization".to_string()]
}

fn default_max_age() -> u64 {
    3600
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit: default_rate_limit(),
            timeout: default_timeout(),
            max_body_size: default_max_body_size(),
        }
    }
}

fn default_rate_limit() -> u32 {
    100
}

fn default_timeout() -> u64 {
    15
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
