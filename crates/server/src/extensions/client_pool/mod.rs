pub mod listener;

use std::{ops::Deref, sync::Arc};

use axum::Extension;
use tondi_grpc_client::{GrpcClient, error::Error as GrpcClientError};
use tondi_scan_library::log::info;
use workflow_rpc::client::{RpcClient, ConnectOptions};

use crate::{
    ctx::event_config::EventType,
    error::{Error, Result},
    extensions::client_pool::listener::ListenerManager,
    shared::pool::{Error as PoolError, HealthCheck, Metadata, Pool},
};

pub enum Client {
    Grpc(GrpcClientWrapper),
    Wrpc(WrpcClientWrapper),
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Client::Grpc(_) => write!(f, "Client::Grpc"),
            Client::Wrpc(_) => write!(f, "Client::Wrpc"),
        }
    }
}

#[derive(Debug)]
pub struct GrpcClientWrapper {
    inner: GrpcClient,
    pub listener_manager: Arc<ListenerManager>,
}

pub struct WrpcClientWrapper {
    inner: Arc<RpcClient<(), workflow_rpc::id::Id64>>,
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
        // Check if the URL starts with ws:// or wss://
        if url.starts_with("ws://") || url.starts_with("wss://") {
            info!("Connecting to wRPC endpoint: {}", url);
            
            // Use wRPC client
            let inner = Arc::new(RpcClient::<(), workflow_rpc::id::Id64>::new::<workflow_rpc::client::JsonProtocol<(), workflow_rpc::id::Id64>>(
                None,
                workflow_rpc::client::Options::default(),
                None
            )?);
            inner.connect(ConnectOptions::default()).await?;
            
            let listener_manager = ListenerManager::new_wrpc(&inner, events).await?;
            
            info!("Successfully connected to wRPC endpoint");
            Ok(Self::Wrpc(WrpcClientWrapper { inner, listener_manager: Arc::new(listener_manager) }))
        } else if url.starts_with("grpc://") || url.starts_with("http://") || url.starts_with("https://") {
            info!("Connecting to gRPC endpoint: {}", url);
            
            // Use gRPC client
            let inner = GrpcClient::connect_with_args(
                tondi_rpc_core::notify::mode::NotificationMode::MultiListeners,
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

            let listener_manager = ListenerManager::new(&inner).await?;

            info!("Successfully connected to gRPC endpoint");
            Ok(Self::Grpc(GrpcClientWrapper { inner, listener_manager: Arc::new(listener_manager) }))
        } else {
            // 尝试自动检测协议
            if url.contains(':') && !url.contains("://") {
                // 可能是IP:PORT格式，默认使用wRPC
                let wrpc_url = format!("ws://{}", url);
                info!("Auto-detected wRPC format, using: {}", wrpc_url);
                Box::pin(Self::connect_with_events(wrpc_url, events)).await
            } else {
                Err(PoolError::from(format!("Unsupported URL format: {}", url)))
            }
        }
    }
    
    pub fn listener_manager(&self) -> &Arc<ListenerManager> {
        match self {
            Client::Grpc(client) => &client.listener_manager,
            Client::Wrpc(client) => &client.listener_manager,
        }
    }
}

impl Deref for GrpcClientWrapper {
    type Target = GrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Deref for WrpcClientWrapper {
    type Target = Arc<RpcClient<(), workflow_rpc::id::Id64>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl WrpcClientWrapper {
    pub fn is_connected(&self) -> bool {
        // wRPC客户端总是返回true，因为连接状态由底层管理
        true
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
        match self {
            Client::Grpc(client) => client.is_connected(),
            Client::Wrpc(client) => client.is_connected(),
        }
    }
}

impl From<GrpcClientError> for PoolError {
    fn from(err: GrpcClientError) -> Self {
        Self::from(format!("Connect Failed: {err}"))
    }
}

impl From<workflow_rpc::client::Error> for PoolError {
    fn from(err: workflow_rpc::client::Error) -> Self {
        Self::from(format!("wRPC Connect Failed: {err}"))
    }
}

impl From<tondi_rpc_core::notify::mode::NotificationMode> for PoolError {
    fn from(err: tondi_rpc_core::notify::mode::NotificationMode) -> Self {
        Self::from(format!("NotificationMode Failed: {err:?}"))
    }
}

impl From<GrpcClientError> for Error {
    fn from(err: GrpcClientError) -> Self {
        Self::from(format!("{err}"))
    }
}

impl From<workflow_rpc::client::Error> for Error {
    fn from(err: workflow_rpc::client::Error) -> Self {
        Self::from(format!("wRPC Error: {err}"))
    }
}

impl From<tondi_rpc_core::notify::mode::NotificationMode> for Error {
    fn from(err: tondi_rpc_core::notify::mode::NotificationMode) -> Self {
        Self::from(format!("NotificationMode Error: {err:?}"))
    }
}

impl From<tondi_rpc_core::RpcError> for PoolError {
    fn from(err: tondi_rpc_core::RpcError) -> Self {
        Self::from(format!("RPC Failed: {err}"))
    }
}

impl From<tondi_rpc_core::RpcError> for Error {
    fn from(err: tondi_rpc_core::RpcError) -> Self {
        Self::from(format!("RPC Failed: {err}"))
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
