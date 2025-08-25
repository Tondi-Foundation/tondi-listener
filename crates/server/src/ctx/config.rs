use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

use crate::ctx::{Context, event_config::{EventConfig, EventStrategy}};
use tondi_listener_library::log::{info, warn};

// Import TONDI related types
use tondi_consensus_core::network::NetworkType;
use workflow_rpc::encoding::Encoding as WrpcEncoding;

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
    #[error("Invalid wRPC configuration: {0}")]
    InvalidWrpcConfig(String),
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
    // Default to allow all origins, equivalent to no CORS restrictions
    vec![]
}

fn default_allowed_methods() -> Vec<String> {
    // Default to allow all methods, equivalent to no CORS restrictions
    vec![]
}

fn default_allowed_headers() -> Vec<String> {
    // Default to allow all headers, equivalent to no CORS restrictions
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
    #[serde(default)]
    pub wrpc: WrpcConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WrpcConfig {
    /// wRPC protocol type: "ws", "wss"
    #[serde(default = "default_wrpc_protocol")]
    pub protocol: String,
    
    /// wRPC node address
    #[serde(default = "default_wrpc_host")]
    pub host: String,
    
    /// wRPC port (if 0, will use default port for network type)
    #[serde(default = "default_wrpc_port")]
    pub port: u16,
    
    /// Network type: "mainnet", "testnet", "devnet", "simnet"
    #[serde(default = "default_wrpc_network")]
    pub network: String,
    
    /// Encoding type: "borsh", "json"
    #[serde(default = "default_wrpc_encoding")]
    pub encoding: String,
    
    /// Whether to enable wRPC (if true, will prioritize wRPC over gRPC)
    #[serde(default = "default_wrpc_enabled")]
    pub enabled: bool,
}

impl Default for WrpcConfig {
    fn default() -> Self {
        Self {
            protocol: default_wrpc_protocol(),
            host: default_wrpc_host(),
            port: default_wrpc_port(),
            network: default_wrpc_network(),
            encoding: default_wrpc_encoding(),
            enabled: default_wrpc_enabled(),
        }
    }
}

fn default_wrpc_protocol() -> String {
    "ws".to_string()
}

fn default_wrpc_host() -> String {
    "8.210.45.192".to_string()
}

fn default_wrpc_port() -> u16 {
    0  // 0 means use default port
}

fn default_wrpc_network() -> String {
    "devnet".to_string()
}

fn default_wrpc_encoding() -> String {
    "borsh".to_string()
}

fn default_wrpc_enabled() -> bool {
    true  // Default to enable wRPC
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_environment() -> String {
    "development".to_string()
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
            wrpc: WrpcConfig::default(),
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
        
        // Load wRPC configuration from environment variables
        if let Ok(protocol) = env::var("TONDI_SCAN_WRPC_PROTOCOL") {
            config.wrpc.protocol = protocol;
        }
        
        if let Ok(host) = env::var("TONDI_SCAN_WRPC_HOST") {
            config.wrpc.host = host;
        }
        
        if let Ok(port) = env::var("TONDI_SCAN_WRPC_PORT") {
            if let Ok(port_num) = port.parse() {
                config.wrpc.port = port_num;
            }
        }
        
        if let Ok(network) = env::var("TONDI_SCAN_WRPC_NETWORK") {
            config.wrpc.network = network;
        }
        
        if let Ok(encoding) = env::var("TONDI_SCAN_WRPC_ENCODING") {
            config.wrpc.encoding = encoding;
        }
        
        if let Ok(enabled) = env::var("TONDI_SCAN_WRPC_ENABLED") {
            config.wrpc.enabled = enabled.parse().unwrap_or(false);
        }
        
        // Validate config
        config.validate()?;
        
        // Log configuration summary
        info!("Configuration loaded successfully:");
        info!("  Environment: {}", config.environment);
        info!("  Log level: {}", config.log_level);
        info!("  Host URL: {}", config.host_url);
        info!("  Database URL: {}", config.database_url);
        info!("  gRPC URL: {}", config.grpc_url);
        info!("  wRPC enabled: {}", config.wrpc.enabled);
        if config.wrpc.enabled {
            info!("  wRPC URL: {}", config.wrpc.build_url());
            info!("  wRPC protocol: {}", config.wrpc.protocol);
            info!("  wRPC network: {}", config.wrpc.network);
            info!("  wRPC encoding: {}", config.wrpc.encoding);
            info!("  wRPC port: {}", config.wrpc.get_port_info());
        }
        
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
        
        // Validate wRPC configuration
        self.wrpc.validate()
            .map_err(|e| ConfigError::InvalidWrpcConfig(e))?;
        
        // Validate wRPC port if specified
        if self.wrpc.port > 0 {
            if self.wrpc.port < 1024 {
                return Err(ConfigError::InvalidPort(self.wrpc.port));
            }
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

impl WrpcConfig {
    /// Build wRPC URL
    pub fn build_url(&self) -> String {
        let port = if self.port == 0 {
            self.get_default_port()
        } else {
            self.port
        };
        
        // Validate port range
        if port < 1024 {
            warn!("wRPC port {} is outside valid range (1024-65535), using default", port);
            let default_port = self.get_default_port();
            format!("{}://{}:{}", self.protocol, self.host, default_port)
        } else {
            format!("{}://{}:{}", self.protocol, self.host, port)
        }
    }
    
    /// Get network type
    pub fn get_network_type(&self) -> Result<NetworkType, String> {
        match self.network.to_lowercase().as_str() {
            "mainnet" => Ok(NetworkType::Mainnet),
            "testnet" => Ok(NetworkType::Testnet),
            "devnet" => Ok(NetworkType::Devnet),
            "simnet" => Ok(NetworkType::Simnet),
            _ => Err(format!("Invalid network type: {}", self.network)),
        }
    }
    
    /// Get encoding type
    pub fn get_encoding(&self) -> Result<WrpcEncoding, String> {
        match self.encoding.to_lowercase().as_str() {
            "borsh" => Ok(WrpcEncoding::Borsh),
            "json" => Ok(WrpcEncoding::SerdeJson),
            _ => Err(format!("Invalid encoding type: {}", self.encoding)),
        }
    }
    
    /// Get default port
    pub fn get_default_port(&self) -> u16 {
        let network_type = self.get_network_type().unwrap_or_else(|_| NetworkType::Devnet);
        let encoding = self.get_encoding().unwrap_or_else(|_| WrpcEncoding::Borsh);
        
        match encoding {
            WrpcEncoding::Borsh => network_type.default_borsh_rpc_port(),
            WrpcEncoding::SerdeJson => network_type.default_json_rpc_port(),
        }
    }
    
    /// Get port info for logging
    pub fn get_port_info(&self) -> String {
        if self.port == 0 {
            format!("{} (auto-detected)", self.get_default_port())
        } else {
            format!("{} (manual)", self.port)
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate protocol type
        match self.protocol.to_lowercase().as_str() {
            "ws" | "wss" => {},
            _ => return Err(format!("Invalid protocol: {}", self.protocol)),
        }
        
        // Validate network type
        self.get_network_type()?;
        
        // Validate encoding type
        self.get_encoding()?;
        
        // Validate host address
        if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }
        
        // Validate port range if specified
        if self.port > 0 && self.port < 1024 {
            return Err(format!("Port {} is outside valid range (1024-65535)", self.port));
        }
        
        Ok(())
    }
}

impl FromRef<Context> for &Config {
    fn from_ref<'a>(ctx: &'a Context) -> Self {
        let this = &*ctx.config;
        // Safety: 'static Context
        unsafe { std::mem::transmute::<&'a Config, Self>(this) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrpc_config_defaults() {
        let config = WrpcConfig::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.protocol, "ws");
        assert_eq!(config.host, "8.210.45.192");
        assert_eq!(config.port, 0);
        assert_eq!(config.network, "devnet");
        assert_eq!(config.encoding, "borsh");
    }

    #[test]
    fn test_wrpc_url_building() {
        let mut config = WrpcConfig::default();
        
        // Test default port (devnet + borsh = 17610)
        let url = config.build_url();
        assert_eq!(url, "ws://8.210.45.192:17610");
        
        // Test custom port
        config.port = 8080;
        let url = config.build_url();
        assert_eq!(url, "ws://8.210.45.192:8080");
        
        // Test different protocol
        config.protocol = "wss".to_string();
        let url = config.build_url();
        assert_eq!(url, "wss://8.210.45.192:8080");
    }

    #[test]
    fn test_wrpc_network_types() {
        let config = WrpcConfig::default();
        
        // Test devnet (default)
        let network = config.get_network_type().unwrap();
        assert_eq!(network, NetworkType::Devnet);
        
        // Test mainnet
        let mut config = WrpcConfig::default();
        config.network = "mainnet".to_string();
        let network = config.get_network_type().unwrap();
        assert_eq!(network, NetworkType::Mainnet);
    }

    #[test]
    fn test_wrpc_encoding_types() {
        let config = WrpcConfig::default();
        
        // Test borsh (default)
        let encoding = config.get_encoding().unwrap();
        assert_eq!(encoding, WrpcEncoding::Borsh);
        
        // Test json
        let mut config = WrpcConfig::default();
        config.encoding = "json".to_string();
        let encoding = config.get_encoding().unwrap();
        assert_eq!(encoding, WrpcEncoding::SerdeJson);
    }

    #[test]
    fn test_wrpc_validation() {
        let config = WrpcConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid protocol
        let mut config = WrpcConfig::default();
        config.protocol = "invalid".to_string();
        assert!(config.validate().is_err());
        
        // Test grpc protocol (should be invalid)
        let mut config = WrpcConfig::default();
        config.protocol = "grpc".to_string();
        assert!(config.validate().is_err());
        
        // Test invalid network
        let mut config = WrpcConfig::default();
        config.network = "invalid".to_string();
        assert!(config.validate().is_err());
        
        // Test invalid encoding
        let mut config = WrpcConfig::default();
        config.encoding = "invalid".to_string();
        assert!(config.validate().is_err());
        
        // Test empty host
        let mut config = WrpcConfig::default();
        config.host = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.wrpc.enabled, true);
        assert_eq!(config.wrpc.protocol, "ws");
        assert_eq!(config.wrpc.host, "8.210.45.192");
        assert_eq!(config.wrpc.network, "devnet");
        assert_eq!(config.wrpc.encoding, "borsh");
    }
}
