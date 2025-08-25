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
use tondi_scan_library::log;

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
        ev: EventType
    ) -> Result<Listener, PoolError> {
        let channel = NotificationChannel::default();
        
        // 实现真正的wRPC订阅逻辑
        // 参考Tondi wasm的实现方式
        let _event_type = match ev {
            EventType::BlockAdded => "block-added",
            EventType::VirtualChainChanged => "virtual-chain-changed",
            EventType::FinalityConflict => "finality-conflict",
            EventType::FinalityConflictResolved => "finality-conflict-resolved",
            EventType::UtxosChanged => "utxos-changed",
            EventType::SinkBlueScoreChanged => "sink-blue-score-changed",
            EventType::VirtualDaaScoreChanged => "virtual-daa-score-changed",
            EventType::PruningPointUtxoSetOverride => "pruning-point-utxo-set-override",
            EventType::NewBlockTemplate => "new-block-template",
        };
        
        // 使用workflow-rpc的订阅机制
        // 这里我们需要根据具体的事件类型来订阅
        // 由于workflow-rpc的API可能需要具体的实现，我们先创建一个基础的订阅框架
        
        // 创建一个唯一的listener ID
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        // TODO: 实现具体的wRPC订阅逻辑
        // 这需要根据workflow-rpc的具体API来实现
        // 可能需要调用类似 client.subscribe(event_name) 的方法
        
        Ok(Self { 
            id,
            channel 
        })
    }
    
    /// 处理wRPC事件通知
    pub async fn handle_wrpc_event(&self, event_data: serde_json::Value) -> Result<(), PoolError> {
        // 将事件数据转换为我们的Notification格式
        // 这里需要根据具体的事件类型来处理
        if let Some(event_type) = event_data.get("type").and_then(|v| v.as_str()) {
            match event_type {
                "block-added" => {
                    // 处理区块添加事件
                    log::info!("Received wRPC block-added event: {:?}", event_data);
                    
                    // TODO: 实现真正的通知创建和发送
                    // 由于类型复杂性，暂时只记录日志
                    // 后续需要根据实际需求来实现
                }
                "virtual-chain-changed" => {
                    // 处理虚拟链变化事件
                    log::info!("Received wRPC virtual-chain-changed event: {:?}", event_data);
                }
                "utxos-changed" => {
                    // 处理UTXO变化事件
                    log::info!("Received wRPC utxos-changed event: {:?}", event_data);
                }
                _ => {
                    // 处理其他事件类型
                    log::warn!("Unhandled wRPC event type: {}", event_type);
                }
            }
        }
        
        Ok(())
    }
    
    /// 启动wRPC事件监听
    pub async fn start_wrpc_listening(&self, _client: &Arc<RpcClient<(), workflow_rpc::id::Id64>>) -> Result<(), PoolError> {
        // 这里需要实现wRPC的事件监听逻辑
        // 可能需要启动一个后台任务来监听WebSocket消息
        
        // 创建一个后台任务来处理wRPC事件
        let _channel_sender = self.channel.sender().clone();
        let _client_clone = _client.clone();
        
        tokio::spawn(async move {
            // 这里应该实现真正的wRPC事件监听
            // 由于workflow-rpc的具体API需要进一步研究，我们先创建一个框架
            
            loop {
                // 模拟事件监听循环
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                // TODO: 实现真正的wRPC事件接收和处理
                // 这需要根据workflow-rpc的具体实现来完成
            }
        });
        
        Ok(())
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
    wrpc_event_handler: Option<WrpcEventHandler>,
}

impl ListenerManager {
    /// Create a new ListenerManager with all event types
    pub async fn new(client: &GrpcClient) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        for ev in EventType::get_all_event_types() {
            let listener = Listener::subscribe(&client, ev).await?;
            listeners.insert(ev, listener);
        }
        Ok(Self { listeners, wrpc_event_handler: None })
    }
    
    /// Create a new ListenerManager for wRPC client
    pub async fn new_wrpc(
        client: &Arc<RpcClient<(), workflow_rpc::id::Id64>>, 
        events: &[EventType]
    ) -> Result<Self, PoolError> {
        let mut listeners = HashMap::new();
        
        // 创建wRPC事件处理器
        let event_handler = WrpcEventHandler::new(client.clone(), events.to_vec());
        
        // 启动事件监听
        event_handler.start_listening().await?;
        
        for ev in events {
            let listener = Listener::subscribe_wrpc(client, *ev).await?;
            listeners.insert(*ev, listener);
        }
        
        Ok(Self { 
            listeners, 
            wrpc_event_handler: Some(event_handler) 
        })
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
    
    /// Handle wRPC event (if this is a wRPC manager)
    pub async fn handle_wrpc_event(&self, event_data: serde_json::Value) -> Result<(), PoolError> {
        if let Some(event_handler) = &self.wrpc_event_handler {
            event_handler.handle_event(event_data).await
        } else {
            Err(PoolError::from("This is not a wRPC ListenerManager".to_string()))
        }
    }
    
    /// Check if this is a wRPC manager
    pub fn is_wrpc(&self) -> bool {
        self.wrpc_event_handler.is_some()
    }
}

/// wRPC事件处理器
pub struct WrpcEventHandler {
    _client: Arc<RpcClient<(), workflow_rpc::id::Id64>>,
    event_types: Vec<EventType>,
}

impl std::fmt::Debug for WrpcEventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrpcEventHandler")
            .field("client", &"Arc<RpcClient<(), workflow_rpc::id::Id64>>")
            .field("event_types", &self.event_types)
            .finish()
    }
}

impl WrpcEventHandler {
    pub fn new(client: Arc<RpcClient<(), workflow_rpc::id::Id64>>, event_types: Vec<EventType>) -> Self {
        Self { _client: client, event_types }
    }
    
    /// 启动事件监听
    pub async fn start_listening(&self) -> Result<(), PoolError> {
        log::info!("Starting wRPC event listening for {} event types", self.event_types.len());
        
        // 这里应该实现真正的wRPC事件监听
        // 由于workflow-rpc的具体API需要进一步研究，我们先创建一个框架
        
        // 为每个事件类型创建监听器
        for event_type in &self.event_types {
            self.subscribe_to_event(*event_type).await?;
        }
        
        Ok(())
    }
    
    /// 订阅特定事件类型
    async fn subscribe_to_event(&self, event_type: EventType) -> Result<(), PoolError> {
        let event_name = match event_type {
            EventType::BlockAdded => "block-added",
            EventType::VirtualChainChanged => "virtual-chain-changed",
            EventType::FinalityConflict => "finality-conflict",
            EventType::FinalityConflictResolved => "finality-conflict-resolved",
            EventType::UtxosChanged => "utxos-changed",
            EventType::SinkBlueScoreChanged => "sink-blue-score-changed",
            EventType::VirtualDaaScoreChanged => "virtual-daa-score-changed",
            EventType::PruningPointUtxoSetOverride => "pruning-point-utxo-set-override",
            EventType::NewBlockTemplate => "new-block-template",
        };
        
        log::info!("Subscribing to wRPC event: {}", event_name);
        
        // TODO: 实现真正的wRPC订阅逻辑
        // 这需要根据workflow-rpc的具体API来实现
        // 可能需要调用类似 client.subscribe(event_name) 的方法
        
        Ok(())
    }
    
    /// 处理接收到的wRPC事件
    pub async fn handle_event(&self, event_data: serde_json::Value) -> Result<(), PoolError> {
        if let Some(event_type) = event_data.get("type").and_then(|v| v.as_str()) {
            log::debug!("Received wRPC event: {}", event_type);
            
            // 这里应该将事件转发给相应的监听器
            // 由于我们还没有完整的实现，先记录日志
            match event_type {
                "block-added" => {
                    log::info!("Block added event received");
                }
                "virtual-chain-changed" => {
                    log::info!("Virtual chain changed event received");
                }
                "utxos-changed" => {
                    log::info!("UTXOs changed event received");
                }
                _ => {
                    log::warn!("Unhandled wRPC event type: {}", event_type);
                }
            }
        }
        
        Ok(())
    }
}
