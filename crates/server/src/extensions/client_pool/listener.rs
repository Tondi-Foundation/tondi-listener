use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::mpsc::{Receiver, Sender};
use workflow_rpc::client::RpcClient;
use workflow_rpc::client::notification::Notification as WrpcNotification;
use workflow_rpc::client::rpc::RpcApi;
use workflow_rpc::error::Error as WrpcError;
use workflow_rpc::id::Id64;
use workflow_rpc::encoding::Encoding;
use workflow_rpc::client::JsonProtocol;
use workflow_rpc::client::BorshProtocol;
use log;

use crate::{
    ctx::event_config::EventType,
    error::{Error as AppError, Result},
    shared::pool::{Error as PoolError, Notification, NotificationChannel},
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
        client: &Arc<RpcClient<(), Id64>>, 
        ev: EventType
    ) -> Result<Listener, PoolError> {
        let channel = NotificationChannel::default();
        
        // 实现wRPC订阅逻辑
        let event_type = match ev {
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
        // 创建一个唯一的listener ID
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        // 记录订阅信息
        log::info!("Subscribing to wRPC event: {} with ID: {}", event_type, id);
        
        // 尝试使用workflow-rpc的订阅机制
        // 注意：workflow-rpc的具体订阅API可能需要根据实际使用情况调整
        // 这里我们创建一个基础的订阅框架，等待后续完善
        
        Ok(Self { 
            id,
            channel 
        })
    }
    
    /// 处理wRPC事件通知
    pub async fn handle_wrpc_event(&self, event_data: serde_json::Value) -> Result<(), PoolError> {
        // 将事件数据转换为我们的Notification格式
        let notification = Notification {
            event_type: "wrpc-event".to_string(),
            data: event_data,
            timestamp: chrono::Utc::now(),
        };
        
        // 发送到通知通道
        if let Err(e) = self.channel.sender().send(notification).await {
            return Err(PoolError::from(format!("Failed to send wRPC event: {}", e)));
        }
        
        Ok(())
    }
    
    /// 启动wRPC事件监听
    pub async fn start_wrpc_listening(&self, client: &Arc<RpcClient<(), Id64>>) -> Result<(), PoolError> {
        // 启动wRPC事件监听逻辑
        let channel_sender = self.channel.sender().clone();
        let client_clone = client.clone();
        
        tokio::spawn(async move {
            log::info!("Starting wRPC event listening loop");
            
            loop {
                // 检查连接状态
                if !client_clone.is_connected() {
                    log::warn!("wRPC client disconnected, attempting to reconnect...");
                    if let Err(e) = client_clone.connect(workflow_rpc::client::ConnectOptions::default()).await {
                        log::error!("Failed to reconnect wRPC client: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                    log::info!("wRPC client reconnected successfully");
                }
                
                // 尝试接收通知
                match client_clone.receive_notification().await {
                    Ok(notification) => {
                        log::debug!("Received wRPC notification: {:?}", notification);
                        
                        // 处理通知
                        if let Err(e) = Self::process_wrpc_notification(notification, &channel_sender).await {
                            log::error!("Failed to process wRPC notification: {}", e);
                        }
                    }
                    Err(e) => {
                        if e.to_string().contains("timeout") {
                            // 超时是正常的，继续循环
                            continue;
                        }
                        log::error!("Error receiving wRPC notification: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
    
    /// 处理wRPC通知
    async fn process_wrpc_notification(
        notification: WrpcNotification<(), Id64>,
        sender: &Sender<Notification>
    ) -> Result<(), PoolError> {
        // 解析通知数据
        let event_data = match notification.payload {
            workflow_rpc::client::notification::Payload::Json(data) => data,
            workflow_rpc::client::notification::Payload::Borsh(_) => {
                // 对于Borsh编码，我们需要先反序列化
                // 这里暂时使用默认值，实际应该根据Borsh格式解析
                serde_json::Value::Null
            }
        };
        
        // 创建通知
        let notification = Notification {
            event_type: "wrpc-event".to_string(),
            data: event_data,
            timestamp: chrono::Utc::now(),
        };
        
        // 发送到通知通道
        sender.send(notification).await
            .map_err(|e| PoolError::from(format!("Failed to send wRPC event: {}", e)))?;
        
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
        client: &Arc<RpcClient<(), Id64>>, 
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
    client: Arc<RpcClient<(), Id64>>,
    event_types: Vec<EventType>,
    listeners: HashMap<EventType, Arc<Listener>>,
}

impl std::fmt::Debug for WrpcEventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrpcEventHandler")
            .field("client", &"Arc<RpcClient<(), Id64>>")
            .field("event_types", &self.event_types)
            .field("listeners", &self.listeners.len())
            .finish()
    }
}

impl WrpcEventHandler {
    pub fn new(
        client: Arc<RpcClient<(), Id64>>, 
        event_types: Vec<EventType>
    ) -> Self {
        Self {
            client,
            event_types,
            listeners: HashMap::new(),
        }
    }
    
    /// 启动事件监听
    pub async fn start_listening(&mut self) -> Result<(), PoolError> {
        // 为每个事件类型创建监听器
        for event_type in &self.event_types {
            let listener = Listener::subscribe_wrpc(&self.client, *event_type).await?;
            self.listeners.insert(*event_type, Arc::new(listener));
        }
        
        // 启动WebSocket消息监听
        self.start_websocket_listening().await?;
        
        Ok(())
    }
    
    /// 启动WebSocket消息监听
    async fn start_websocket_listening(&self) -> Result<(), PoolError> {
        let client = self.client.clone();
        let listeners = self.listeners.clone();
        
        tokio::spawn(async move {
            loop {
                // 检查连接状态
                if !client.is_connected() {
                    log::warn!("wRPC client disconnected, attempting to reconnect...");
                    if let Err(e) = client.connect(workflow_rpc::client::ConnectOptions::default()).await {
                        log::error!("Failed to reconnect wRPC client: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }
                    log::info!("wRPC client reconnected successfully");
                }
                
                // 监听WebSocket消息
                if let Ok(notification) = client.receive_notification().await {
                    Self::handle_notification(notification, &listeners).await;
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
    
    /// 处理接收到的通知
    async fn handle_notification(
        notification: WrpcNotification<(), Id64>,
        listeners: &HashMap<EventType, Arc<Listener>>
    ) {
        // 解析通知数据
        let event_data = match notification.payload {
            workflow_rpc::client::notification::Payload::Json(data) => data,
            workflow_rpc::client::notification::Payload::Borsh(_) => {
                // 对于Borsh编码，我们需要先反序列化
                // 这里暂时使用默认值，实际应该根据Borsh格式解析
                serde_json::Value::Null
            }
        };
        
        log::debug!("Received wRPC notification: {:?}", notification);
        
        // 尝试解析事件类型
        let event_type = event_data.get("type")
            .and_then(|v| v.as_str());
        
        if let Some(event_type_str) = event_type {
            // 根据事件类型找到对应的监听器
            let event_enum = match event_type_str {
                "block-added" => EventType::BlockAdded,
                "virtual-chain-changed" => EventType::VirtualChainChanged,
                "finality-conflict" => EventType::FinalityConflict,
                "finality-conflict-resolved" => EventType::FinalityConflictResolved,
                "utxos-changed" => EventType::UtxosChanged,
                "sink-blue-score-changed" => EventType::SinkBlueScoreChanged,
                "virtual-daa-score-changed" => EventType::VirtualDaaScoreChanged,
                "pruning-point-utxo-set-override" => EventType::PruningPointUtxoSetOverride,
                "new-block-template" => EventType::NewBlockTemplate,
                _ => {
                    log::warn!("Unknown event type: {}", event_type_str);
                    return;
                }
            };
            
            // 发送到对应的监听器
            if let Some(listener) = listeners.get(&event_enum) {
                if let Err(e) = listener.handle_wrpc_event(event_data).await {
                    log::error!("Failed to handle wRPC event: {}", e);
                }
            } else {
                log::warn!("No listener found for event type: {}", event_type_str);
            }
        } else {
            log::warn!("No event type found in wRPC notification");
        }
    }
    
    /// 处理事件
    pub async fn handle_event(&self, event_data: serde_json::Value) -> Result<(), PoolError> {
        // 解析事件类型
        let event_type = event_data.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PoolError::from("Missing event type"))?;
        
        // 根据事件类型找到对应的监听器
        let event_enum = match event_type {
            "block-added" => EventType::BlockAdded,
            "virtual-chain-changed" => EventType::VirtualChainChanged,
            "finality-conflict" => EventType::FinalityConflict,
            "finality-conflict-resolved" => EventType::FinalityConflictResolved,
            "utxos-changed" => EventType::UtxosChanged,
            "sink-blue-score-changed" => EventType::SinkBlueScoreChanged,
            "virtual-daa-score-changed" => EventType::VirtualDaaScoreChanged,
            "pruning-point-utxo-set-override" => EventType::PruningPointUtxoSetOverride,
            "new-block-template" => EventType::NewBlockTemplate,
            _ => {
                log::warn!("Unknown event type: {}", event_type);
                return Ok(());
            }
        };
        
        if let Some(listener) = self.listeners.get(&event_enum) {
            listener.handle_wrpc_event(event_data).await?;
        }
        
        Ok(())
    }
}
