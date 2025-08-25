use std::{collections::HashMap, ops::Deref, sync::Arc};

use tondi_grpc_client::GrpcClient;
use tondi_grpc_core::channel::NotificationChannel;
use tondi_notify::{
    connection::ChannelType,
    events::EventType as TondiEventType,
};
use tondi_rpc_core::{Notification, api::rpc::RpcApi, notify::connection::ChannelConnection};
use tondi_utils::channel::Receiver;
use workflow_rpc::client::RpcClient;

use crate::{
    ctx::event_config::EventType,
    error::{Result, Error as AppError},
    shared::pool::Error as PoolError,
};

#[derive(Debug)]
pub struct Listener {
    pub id: u64,
    pub channel: NotificationChannel,
}

impl Listener {
    pub async fn subscribe(client: &GrpcClient, ev: EventType) -> Result<Listener, PoolError> {
        let channel = NotificationChannel::default();
        let conn = ChannelConnection::new("Listener", channel.sender(), ChannelType::Closable);
        let id = client.register_new_listener(conn);
        
        // Convert our EventType to Tondi's EventType
        let tondi_event: TondiEventType = ev.into();
        client.start_notify(id, tondi_event.into()).await?;
        Ok(Self { id, channel })
    }
    
    pub async fn subscribe_wrpc(
        _client: &Arc<RpcClient<(), workflow_rpc::id::Id64>>, 
        _ev: EventType
    ) -> Result<Listener, PoolError> {
        let channel = NotificationChannel::default();
        
        // TODO: 实现真正的wRPC订阅逻辑
        Ok(Self { 
            id: 0, // wRPC不需要listener ID
            channel 
        })
    }
}

impl Deref for Listener {
    type Target = NotificationChannel;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

// Convert our EventType to Tondi's EventType
impl From<EventType> for TondiEventType {
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::BlockAdded => TondiEventType::BlockAdded,
            EventType::VirtualChainChanged => TondiEventType::VirtualChainChanged,
            EventType::FinalityConflict => TondiEventType::FinalityConflict,
            EventType::FinalityConflictResolved => TondiEventType::FinalityConflictResolved,
            EventType::UtxosChanged => TondiEventType::UtxosChanged,
            EventType::SinkBlueScoreChanged => TondiEventType::SinkBlueScoreChanged,
            EventType::VirtualDaaScoreChanged => TondiEventType::VirtualDaaScoreChanged,
            EventType::PruningPointUtxoSetOverride => TondiEventType::PruningPointUtxoSetOverride,
            EventType::NewBlockTemplate => TondiEventType::NewBlockTemplate,
        }
    }
}

#[derive(Debug)]
pub struct ListenerManager {
    listeners: HashMap<EventType, Listener>,
}

impl ListenerManager {
    /// Create a new ListenerManager with all event types
    pub async fn new(client: &GrpcClient) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for ev in EventType::get_all_event_types() {
            let listener = Listener::subscribe(&client, ev).await?;
            listeners.insert(ev, listener);
        }
        Ok(Self { listeners })
    }
    
    /// Create a new ListenerManager for wRPC client
    pub async fn new_wrpc(
        client: &Arc<RpcClient<(), workflow_rpc::id::Id64>>, 
        _events: &[EventType]
    ) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for ev in EventType::get_all_event_types() {
            let listener = Listener::subscribe_wrpc(&client, ev).await?;
            listeners.insert(ev, listener);
        }
        Ok(Self { listeners })
    }

    /// Get receiver for a specific event type
    pub fn get(&self, ev: &EventType) -> Result<Receiver<Notification>> {
        match self.listeners.get(ev) {
            Some(listener) => Ok(listener.receiver()),
            None => Err(AppError::NotFound("EventType not found".to_string())),
        }
    }

    /// Check if an event type is being listened to
    pub fn has_event(&self, ev: &EventType) -> bool {
        self.listeners.contains_key(ev)
    }

    /// Get all active event types
    pub fn get_active_events(&self) -> Vec<EventType> {
        self.listeners.keys().cloned().collect()
    }

    /// Get listener count
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
}
