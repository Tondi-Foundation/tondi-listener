use axum::extract::FromRef;
use serde::{Deserialize, Serialize};

use crate::ctx::Context;

const HOST_URL: &str = "127.0.0.1:3003";

const GRPC_URL: &str = "grpc://8.210.45.192:16610";

const DATABASE_URL: &str = "postgres://postgres:postgres@127.0.0.1/postgres";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub host_url: String,
    pub grpc_url: String,
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host_url: HOST_URL.into(),
            grpc_url: GRPC_URL.into(),
            database_url: DATABASE_URL.into(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FromRef<Context> for &Config {
    fn from_ref<'a>(ctx: &'a Context) -> Self {
        let this = &*ctx.config;
        // Safety: 'static Context
        unsafe { std::mem::transmute::<&'a Config, Self>(this) }
    }
}
