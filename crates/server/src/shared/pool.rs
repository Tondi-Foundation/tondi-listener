use std::fmt::Debug as StdDebug;

use tokio::sync::{RwLock, RwLockReadGuard, TryLockError};

pub trait HealthCheck {
    fn is_live(&self) -> bool;
}

pub trait Metadata: Sized {
    type Meta;

    type Error;

    fn try_from(metadata: &Self::Meta) -> impl Future<Output = Result<Self, Self::Error>>;
}

#[derive(Debug)]
pub struct Pool<T>
where
    T: Metadata,
{
    meta: T::Meta,
    // TODO: Multi
    pool: RwLock<T>,
}

impl<T> Pool<T>
where
    T: Metadata,
    T: StdDebug,
    T: HealthCheck,
    Error: From<T::Error>,
{
    pub fn new(meta: T::Meta, init: T) -> Self {
        Self { meta, pool: RwLock::new(init) }
    }

    pub async fn get(&self) -> Result<RwLockReadGuard<'_, T>, Error> {
        let Self { meta, pool } = self;
        // Read
        {
            let elm = pool.try_read()?;
            if elm.is_live() {
                return Ok(elm)
            }
        }
        // Refresh
        {
            let mut elm = pool.write().await;
            *elm = T::try_from(meta).await?;
        }
        Ok(pool.try_read()?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TryLockError(#[from] TryLockError),

    #[error("{0}")]
    PoolError(String),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::PoolError(err)
    }
}
