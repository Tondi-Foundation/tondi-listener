pub mod consumer;
pub mod listener;

use std::{ops::Deref, sync::Arc};

use axum::Extension;
use tondi_grpc_client::{GrpcClient, error::Error as GrpcClientError};
use tondi_rpc_core::{RpcError, notify::mode::NotificationMode};

use crate::{
    error::{Error, Result},
    extensions::client_pool::consumer::Consumer,
    shared::pool::{Error as PoolError, HealthCheck, Metadata, Pool},
};

#[derive(Debug)]
pub struct Client {
    inner: GrpcClient,
    pub consumer: Consumer,
}

impl Client {
    pub const fn new(inner: GrpcClient, consumer: Consumer) -> Self {
        Self { inner, consumer }
    }

    pub async fn client_multi(url: String) -> Result<GrpcClient, PoolError> {
        let inner = GrpcClient::connect_with_args(
            NotificationMode::MultiListeners,
            url,
            None,
            true,
            None,
            false,
            None,
            Default::default(),
        )
        .await?;
        inner.start(None).await;
        Ok(inner)
    }

    pub async fn client_direct(url: String) -> Result<GrpcClient, PoolError> {
        let inner = GrpcClient::connect(url).await?;
        inner.start(None).await;
        Ok(inner)
    }

    pub async fn connect(url: String) -> Result<Self, PoolError> {
        // let inner = Self::client_multi(url).await?;
        let inner = Self::client_direct(url).await?;
        let listener_manager = Consumer::new(&inner).await?;
        Ok(Self::new(inner, listener_manager))
    }
}

impl Deref for Client {
    type Target = GrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Metadata for Client {
    type Error = PoolError;
    type Meta = String;

    async fn try_from(url: &Self::Meta) -> Result<Self, Self::Error> {
        Ok(Self::connect(url.clone()).await?)
    }
}

impl HealthCheck for Client {
    fn is_live(&self) -> bool {
        self.is_connected()
    }
}

impl From<GrpcClientError> for PoolError {
    fn from(err: GrpcClientError) -> Self {
        Self::from(format!("Connect Fail: {err}"))
    }
}

impl From<RpcError> for PoolError {
    fn from(err: RpcError) -> Self {
        Self::from(format!("RPC Fail: {err}"))
    }
}

impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Self {
        Self::from(format!("{err}"))
    }
}

impl From<RpcError> for Error {
    fn from(err: RpcError) -> Self {
        Self::from(format!("{err}"))
    }
}

pub type ClientPool = Extension<Arc<Pool<Client>>>;

pub async fn extension(url: &String) -> Result<ClientPool, PoolError> {
    let client = Client::connect(url.into()).await?;
    let pool = Pool::new(url.into(), client);
    Ok(Extension(Arc::new(pool)))
}
