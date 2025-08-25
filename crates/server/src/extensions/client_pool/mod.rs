pub mod listener;

use std::{ops::Deref, sync::Arc};

use axum::Extension;
use tondi_grpc_client::{GrpcClient, error::Error as GrpcClientError};
use tondi_rpc_core::{RpcError, notify::mode::NotificationMode};

use crate::{
    ctx::event_config::EventType,
    error::{Error, Result},
    extensions::client_pool::listener::ListenerManager,
    shared::pool::{Error as PoolError, HealthCheck, Metadata, Pool},
};

#[derive(Debug)]
pub struct Client {
    inner: GrpcClient,
    pub listener_manager: Arc<ListenerManager>,
}

impl Client {
    pub async fn connect(url: String) -> Result<Self, PoolError> {
        Self::connect_with_events(url, &[]).await
    }

    pub async fn connect_with_events(
        url: String, 
        events: &[EventType]
    ) -> Result<Self, PoolError> {
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

        let listener_manager = if events.is_empty() {
            // If no events specified, use all events (legacy behavior)
            ListenerManager::new(&inner).await?
        } else {
            // Use only specified events
            ListenerManager::new_with_events(&inner, events).await?
        };

        Ok(Self { inner, listener_manager: Arc::new(listener_manager) })
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
        Self::from(format!("Connect Failed: {err}"))
    }
}

impl From<RpcError> for PoolError {
    fn from(err: RpcError) -> Self {
        Self::from(format!("RPC Failed: {err}"))
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
    extension_with_events(url, &[]).await
}

pub async fn extension_with_events(
    url: &String, 
    events: &[EventType]
) -> Result<ClientPool, PoolError> {
    let client = Client::connect_with_events(url.into(), events).await?;
    let pool = Pool::new(url.into(), client);
    Ok(Extension(Arc::new(pool)))
}
