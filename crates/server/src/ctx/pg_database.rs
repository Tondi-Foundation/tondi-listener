use std::ops::Deref;

use axum::extract::{FromRef, State};
use xscan_db::diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PoolError},
};

use crate::ctx::Context;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub struct PgDatabase {
    pool: PgPool,
}

impl PgDatabase {
    pub fn new(url: &str) -> Result<Self, PoolError> {
        let manager = ConnectionManager::new(url);
        let pool = Pool::builder().build(manager)?;
        Ok(Self { pool })
    }
}

impl Deref for PgDatabase {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl FromRef<Context> for &PgDatabase {
    fn from_ref<'a>(ctx: &'a Context) -> Self {
        let this = &*ctx.pg_database;
        // Safety: 'static Context
        unsafe { std::mem::transmute::<&'a PgDatabase, Self>(this) }
    }
}

pub type PgDb<'a> = State<&'a PgDatabase>;
