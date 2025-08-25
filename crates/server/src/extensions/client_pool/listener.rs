use std::{collections::HashMap, ops::Deref};

use tondi_grpc_client::GrpcClient;
use tondi_grpc_core::channel::NotificationChannel;
use tondi_notify::{
    connection::ChannelType,
    events::EventType as TondiEventType,
};
use tondi_rpc_core::{Notification, api::rpc::RpcApi, notify::connection::ChannelConnection};
use tondi_utils::channel::Receiver;

use crate::{
    ctx::event_config::EventType,
    error::{Result, err},
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
    /// Create a new ListenerManager with all event types (legacy behavior)
    pub async fn new(client: &GrpcClient) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for ev in EventType::get_all_event_types() {
            let listener = Listener::subscribe(&client, ev).await?;
            listeners.insert(ev, listener);
        }
        Ok(Self { listeners })
    }

    /// Create a new ListenerManager with only specified event types
    pub async fn new_with_events(
        client: &GrpcClient, 
        events: &[EventType]
    ) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for &event_type in events {
            let listener = Listener::subscribe(&client, event_type).await?;
            listeners.insert(event_type, listener);
        }
        Ok(Self { listeners })
    }

    /// Subscribe to additional events
    pub async fn subscribe_to_events(
        &mut self, 
        client: &GrpcClient, 
        events: &[EventType]
    ) -> Result<(), PoolError> {
        for &event_type in events {
            if !self.listeners.contains_key(&event_type) {
                let listener = Listener::subscribe(&client, event_type).await?;
                self.listeners.insert(event_type, listener);
            }
        }
        Ok(())
    }
    
    /// Unsubscribe from specific events
    pub async fn unsubscribe_from_events(
        &mut self, 
        client: &GrpcClient, 
        events: &[EventType]
    ) -> Result<(), PoolError> {
        for &event_type in events {
            if let Some(listener) = self.listeners.remove(&event_type) {
                client.unregister_listener(listener.id).await?;
            }
        }
        Ok(())
    }

    /// Get receiver for a specific event type
    pub fn get(&self, ev: &EventType) -> Result<Receiver<Notification>> {
        match self.listeners.get(ev) {
            Some(listener) => Ok(listener.receiver()),
            None => err!("EventType {ev} Not Found"),
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
