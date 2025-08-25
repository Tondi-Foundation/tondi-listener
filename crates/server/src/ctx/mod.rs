pub mod config;
pub mod event_config;
pub mod pg_database;

use std::sync::Arc;

use crate::{
    ctx::{config::Config, pg_database::PgDatabase},
    error::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct Context {
    pub config: Arc<Config>,
    pub pg_database: Arc<PgDatabase>,
}

impl Context {
    /// Create new Context from environment variables
    pub fn from_env() -> Result<Self> {
        let config = Config::from_env()
            .map_err(|e| Error::Config(e))?;
        
        Self::new(config)
    }
    
    /// Create new Context with specified configuration
    pub fn new(config: Config) -> Result<Self> {
        let pg_database = PgDatabase::new(&config.database_url)?;
        Ok(Self { 
            config: Arc::new(config), 
            pg_database: Arc::new(pg_database) 
        })
    }
    
    /// Check if production environment
    pub fn is_production(&self) -> bool {
        self.config.is_production()
    }
    
    /// Check if development environment
    pub fn is_development(&self) -> bool {
        self.config.is_development()
    }
    
    /// Get log level
    pub fn log_level(&self) -> &str {
        &self.config.log_level
    }
    
    /// Get security configuration
    pub fn security_config(&self) -> &crate::ctx::config::SecurityConfig {
        &self.config.security
    }
    
    /// Get CORS configuration
    pub fn cors_config(&self) -> &crate::ctx::config::CorsConfig {
        &self.config.cors
    }
}
