use std::{collections::HashMap, ops::Deref};

use tondi_grpc_client::GrpcClient;
use tondi_grpc_core::channel::NotificationChannel;
use tondi_notify::{
    connection::ChannelType,
    events::{EVENT_TYPE_ARRAY, EventType},
};
use tondi_rpc_core::{Notification, api::rpc::RpcApi, notify::connection::ChannelConnection};
use tondi_utils::channel::Receiver;

use crate::{
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
        client.start_notify(id, ev.into()).await?;
        Ok(Self { id, channel })
    }
}

impl Deref for Listener {
    type Target = NotificationChannel;

    fn deref(&self) -> &Self::Target {
        &self.channel
    }
}

#[derive(Debug)]
pub struct ListenerManager {
    listeners: HashMap<EventType, Listener>,
}

impl ListenerManager {
    pub async fn new(client: &GrpcClient) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for ev in EVENT_TYPE_ARRAY {
            let listener = Listener::subscribe(&client, ev).await?;
            listeners.insert(ev, listener);
        }
        Ok(Self { listeners })
    }

    pub fn get(&self, ev: &EventType) -> Result<Receiver<Notification>> {
        match self.listeners.get(ev) {
            Some(listener) => Ok(listener.receiver()),
            None => err!("EventType {ev} Not Found"),
        }
    }
}
