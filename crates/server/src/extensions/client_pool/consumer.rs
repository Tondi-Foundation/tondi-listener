use tondi_grpc_client::GrpcClient;
use tondi_notify::events::EVENT_TYPE_ARRAY;
use tondi_rpc_core::{Notification, api::rpc::RpcApi};
use tondi_utils::channel::Receiver;

use crate::{error::Result, shared::pool::Error as PoolError};

#[derive(Debug)]
pub struct Consumer {
    pub rx: Receiver<Notification>,
}

impl Consumer {
    pub async fn new(client: &GrpcClient) -> Result<Self, PoolError> {
        for ev in EVENT_TYPE_ARRAY {
            client.start_notify(GrpcClient::DIRECT_MODE_LISTENER_ID, ev.into()).await?;
        }
        let rx = client.notification_channel_receiver();
        Ok(Self { rx })
    }
}
