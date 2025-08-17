pub mod config;
pub mod pg_database;

use std::sync::Arc;

use crate::{
    ctx::{config::Config, pg_database::PgDatabase},
    error::Result,
};

#[derive(Debug, Clone)]
pub struct Context {
    pub config: Arc<Config>,
    pub pg_database: Arc<PgDatabase>,
}

impl Context {
    pub fn new(config: Config) -> Result<Self> {
        let pg_database = PgDatabase::new(&config.database_url)?;
        Ok(Self { config: Arc::new(config), pg_database: Arc::new(pg_database) })
    }
}
