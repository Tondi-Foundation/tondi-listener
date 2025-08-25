use axum::extract::{FromRef, State};
use tondi_scan_db::diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

use crate::{error::Result, ctx::Context};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub struct PgDatabase {
    pool: PgPool,
}

impl PgDatabase {
    pub fn new(url: &str) -> Result<Self> {
        let manager = ConnectionManager::new(url);
        let pool = Pool::builder().build(manager)?;
        Ok(Self { pool })
    }
    
    pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        Ok(self.pool.get()?)
    }
}

impl std::ops::Deref for PgDatabase {
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
