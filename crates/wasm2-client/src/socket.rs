use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::{WebSocket, MessageEvent, ErrorEvent};
use wasm_bindgen::JsCast;
use log;
use std::collections::HashMap;

/// wRPC Client Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcConfig {
    pub url: String,
    pub encoding: String,
    pub network: String,
    pub reconnect_attempts: u32,
    pub reconnect_delay_ms: u32,
}

impl Default for WrpcConfig {
    fn default() -> Self {
        Self {
            url: "ws://8.210.45.192:18610".to_string(),
            encoding: "json".to_string(),
            network: "devnet".to_string(),
            reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
        }
    }
}

/// wRPC Event Type Enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrpcEventType {
    // Core blockchain events
    BlockAdded,
    VirtualChainChanged,
    FinalityConflict,
    FinalityConflictResolved,
    UtxosChanged,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
    
    // Tondi-specific events
    SubnetworkChanged,
    DagStructureChanged,
    ParallelBlockProcessed,
    VirtualChainMerged,
    ConsensusStateChanged,
    NetworkDifficultyChanged,
    MempoolStateChanged,
    PeerConnectionChanged,
}

impl WrpcEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Core blockchain events
            WrpcEventType::BlockAdded => "block-added",
            WrpcEventType::VirtualChainChanged => "virtual-chain-changed",
            WrpcEventType::FinalityConflict => "finality-conflict",
            WrpcEventType::FinalityConflictResolved => "finality-conflict-resolved",
            WrpcEventType::UtxosChanged => "utxos-changed",
            WrpcEventType::SinkBlueScoreChanged => "sink-blue-score-changed",
            WrpcEventType::VirtualDaaScoreChanged => "virtual-daa-score-changed",
            WrpcEventType::PruningPointUtxoSetOverride => "pruning-point-utxo-set-override",
            WrpcEventType::NewBlockTemplate => "new-block-template",
            
            // Tondi-specific events
            WrpcEventType::SubnetworkChanged => "subnetwork-changed",
            WrpcEventType::DagStructureChanged => "dag-structure-changed",
            WrpcEventType::ParallelBlockProcessed => "parallel-block-processed",
            WrpcEventType::VirtualChainMerged => "virtual-chain-merged",
            WrpcEventType::ConsensusStateChanged => "consensus-state-changed",
            WrpcEventType::NetworkDifficultyChanged => "network-difficulty-changed",
            WrpcEventType::MempoolStateChanged => "mempool-state-changed",
            WrpcEventType::PeerConnectionChanged => "peer-connection-changed",
        }
    }
}

/// wRPC Event Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcEvent {
    pub event_type: String,
    pub data: Value,
    pub timestamp: u64,
}

/// wRPC Response Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcResponse {
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

/// wRPC Client Struct
pub struct WrpcClient {
    websocket: Option<WebSocket>,
    config: WrpcConfig,
    event_handlers: HashMap<String, js_sys::Function>,
    pending_requests: HashMap<u64, js_sys::Function>,
    connected: bool,
    reconnect_attempts: u32,
    current_reconnect_attempt: u32,
}

impl WrpcClient {
    /// Create new wRPC Client
    pub fn new(config: WrpcConfig) -> Result<Self, JsValue> {
        Ok(Self {
            websocket: None,
            config,
            event_handlers: HashMap::new(),
            pending_requests: HashMap::new(),
            connected: false,
            reconnect_attempts: 0,
            current_reconnect_attempt: 0,
        })
    }
    
    /// Connect to wRPC Server
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        log::info!("Connecting to wRPC server: {}", self.config.url);
        
        // Create WebSocket Connection
        let websocket = WebSocket::new(&self.config.url)?;
        
        // Set Event Handler
        let event_handlers = self.event_handlers.clone();
        let pending_requests = self.pending_requests.clone();
        
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(text) = event.data().dyn_into::<js_sys::JsString>().ok().and_then(|s| s.as_string()) {
                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    log::debug!("Received WebSocket message: {:?}", data);
                    
                    // Handle different message types
                    if let Some(method) = data.get("method").and_then(|m| m.as_str()) {
                        // This is an event notification
                        if let Some(handler) = event_handlers.get(method) {
                            let _ = handler.call1(&wasm_bindgen::JsValue::NULL, &serde_wasm_bindgen::to_value(&data).unwrap_or_default());
                        }
                    } else if let Some(id) = data.get("id").and_then(|i| i.as_u64()) {
                        // This is a response to an RPC call
                        if let Some(callback) = pending_requests.get(&id) {
                            let _ = callback.call1(&wasm_bindgen::JsValue::NULL, &serde_wasm_bindgen::to_value(&data).unwrap_or_default());
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        
        let onopen_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            log::info!("WebSocket connection opened");
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        let onclose_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            log::info!("WebSocket connection closed");
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        let onerror_callback = Closure::wrap(Box::new(move |_event: ErrorEvent| {
            log::error!("WebSocket error occurred");
        }) as Box<dyn FnMut(ErrorEvent)>);
        
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        
        // Keep Callback Lifecycle
        onmessage_callback.forget();
        onopen_callback.forget();
        onclose_callback.forget();
        onerror_callback.forget();
        
        self.websocket = Some(websocket);
        self.connected = true;
        self.current_reconnect_attempt = 0;
        
        log::info!("Successfully connected to wRPC server");
        Ok(())
    }
    
    /// Disconnect from wRPC Server
    pub async fn disconnect(&mut self) -> Result<(), JsValue> {
        log::info!("Disconnecting from wRPC server");
        
        if let Some(websocket) = &self.websocket {
            let _ = websocket.close();
        }
        
        self.websocket = None;
        self.connected = false;
        
        Ok(())
    }
    
    /// Check Connection Status
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Subscribe to Event
    pub async fn subscribe(&mut self, event_type: WrpcEventType, handler: js_sys::Function) -> Result<(), JsValue> {
        let event_name = event_type.as_str();
        
        log::info!("Subscribing to event: {}", event_name);
        
        // Store Event Handler
        self.event_handlers.insert(event_name.to_string(), handler);
        
        // Send Subscribe Message
        if let Some(websocket) = &self.websocket {
            let subscribe_msg = serde_json::json!({
                "method": "subscribe",
                "params": {
                    "event": event_name
                }
            });
            
            let msg_str = serde_json::to_string(&subscribe_msg)
                .map_err(|e| format!("Failed to serialize subscribe message: {}", e))?;
            
            websocket.send_with_str(&msg_str)?;
        }
        
        Ok(())
    }
    
    /// Send RPC Call with Response Handling
    pub async fn call<Request>(&mut self, method: &str, request: Request, callback: js_sys::Function) -> Result<(), JsValue>
    where
        Request: serde::Serialize + 'static,
    {
        log::debug!("Making RPC call to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let request_id = js_sys::Date::now() as u64;
            
            // Store callback for response handling
            self.pending_requests.insert(request_id, callback);
            
            let call_msg = serde_json::json!({
                "method": method,
                "params": request,
                "id": request_id
            });
            
            let msg_str = serde_json::to_string(&call_msg)
                .map_err(|e| format!("Failed to serialize call message: {}", e))?;
            
            websocket.send_with_str(&msg_str)?;
            Ok(())
        } else {
            Err("WebSocket not connected".into())
        }
    }
    
    /// Send RPC Call (Legacy method for backward compatibility)
    pub async fn call_simple<Request>(&self, method: &str, request: Request) -> Result<Value, JsValue>
    where
        Request: serde::Serialize + 'static,
    {
        log::debug!("Making simple RPC call to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let call_msg = serde_json::json!({
                "method": method,
                "params": request,
                "id": js_sys::Date::now() as u64
            });
            
            let msg_str = serde_json::to_string(&call_msg)
                .map_err(|e| format!("Failed to serialize call message: {}", e))?;
            
            websocket.send_with_str(&msg_str)?;
            
            // Return a placeholder response
            Ok(serde_json::json!({"status": "sent", "note": "Use call() method for response handling"}))
        } else {
            Err("WebSocket not connected".into())
        }
    }
    
    /// Send Notification
    pub async fn notify<Request>(&self, method: &str, request: Request) -> Result<(), JsValue>
    where
        Request: serde::Serialize + 'static,
    {
        log::debug!("Sending notification to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let notify_msg = serde_json::json!({
                "method": method,
                "params": request
            });
            
            let msg_str = serde_json::to_string(&notify_msg)
                .map_err(|e| format!("Failed to serialize notify message: {}", e))?;
            
            websocket.send_with_str(&msg_str)?;
            Ok(())
        } else {
            Err("WebSocket not connected".into())
        }
    }
    
    /// Attempt to reconnect
    pub async fn reconnect(&mut self) -> Result<(), JsValue> {
        if self.current_reconnect_attempt >= self.config.reconnect_attempts {
            return Err("Max reconnection attempts reached".into());
        }
        
        self.current_reconnect_attempt += 1;
        log::info!("Attempting to reconnect (attempt {}/{})", self.current_reconnect_attempt, self.config.reconnect_attempts);
        
        // Wait before attempting reconnection
        let delay = std::time::Duration::from_millis(self.config.reconnect_delay_ms as u64);
        std::thread::sleep(delay);
        
        self.connect().await
    }
    
    /// Clear pending requests
    pub fn clear_pending_requests(&mut self) {
        self.pending_requests.clear();
    }
    
    /// Tondi-specific RPC methods
    
    /// Get subnetwork information
    pub async fn get_subnetwork(&mut self, subnetwork_id: u32, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetSubnetwork", serde_json::json!({ "subnetwork_id": subnetwork_id }), callback).await
    }
    
    /// Get virtual chain from block
    pub async fn get_virtual_chain_from_block(&mut self, block_hash: &str, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetVirtualChainFromBlock", serde_json::json!({ "block_hash": block_hash }), callback).await
    }
    
    /// Get block DAG information
    pub async fn get_block_dag_info(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetBlockDagInfo", serde_json::json!({}), callback).await
    }
    
    /// Get sink information
    pub async fn get_sink(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetSink", serde_json::json!({}), callback).await
    }
    
    /// Get mempool entries by addresses
    pub async fn get_mempool_entries_by_addresses(&mut self, addresses: Vec<String>, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetMempoolEntriesByAddresses", serde_json::json!({ "addresses": addresses }), callback).await
    }
    
    /// Get UTXOs by addresses
    pub async fn get_utxos_by_addresses(&mut self, addresses: Vec<String>, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetUtxosByAddresses", serde_json::json!({ "addresses": addresses }), callback).await
    }
    
    /// Get balance by address
    pub async fn get_balance_by_address(&mut self, address: &str, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetBalanceByAddress", serde_json::json!({ "address": address }), callback).await
    }
    
    /// Get balances by addresses
    pub async fn get_balances_by_addresses(&mut self, addresses: Vec<String>, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetBalancesByAddresses", serde_json::json!({ "addresses": addresses }), callback).await
    }
    
    /// Get coin supply
    pub async fn get_coin_supply(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("GetCoinSupply", serde_json::json!({}), callback).await
    }
    
    /// Estimate network hashes per second
    pub async fn estimate_network_hashes_per_second(&mut self, window_size: u32, callback: js_sys::Function) -> Result<(), JsValue> {
        self.call("EstimateNetworkHashesPerSecond", serde_json::json!({ "window_size": window_size }), callback).await
    }
}

/// JavaScript Binding
#[wasm_bindgen]
pub struct WrpcClientJs {
    inner: WrpcClient,
}

#[wasm_bindgen]
impl WrpcClientJs {
    /// Create new wRPC Client
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WrpcClientJs, JsValue> {
        let config: WrpcConfig = serde_wasm_bindgen::from_value(config)?;
        let inner = WrpcClient::new(config)?;
        Ok(Self { inner })
    }
    
    /// Connect to Server
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.inner.connect().await
    }
    
    /// Disconnect from Server
    pub async fn disconnect(&mut self) -> Result<(), JsValue> {
        self.inner.disconnect().await
    }
    
    /// Check Connection Status
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    /// Subscribe to Event
    pub async fn subscribe(&mut self, event_type: &str, handler: js_sys::Function) -> Result<(), JsValue> {
        let event_enum = match event_type {
            "block-added" => WrpcEventType::BlockAdded,
            "virtual-chain-changed" => WrpcEventType::VirtualChainChanged,
            "finality-conflict" => WrpcEventType::FinalityConflict,
            "finality-conflict-resolved" => WrpcEventType::FinalityConflictResolved,
            "utxos-changed" => WrpcEventType::UtxosChanged,
            "sink-blue-score-changed" => WrpcEventType::SinkBlueScoreChanged,
            "virtual-daa-score-changed" => WrpcEventType::VirtualDaaScoreChanged,
            "pruning-point-utxo-set-override" => WrpcEventType::PruningPointUtxoSetOverride,
            "new-block-template" => WrpcEventType::NewBlockTemplate,
            "subnetwork-changed" => WrpcEventType::SubnetworkChanged,
            "dag-structure-changed" => WrpcEventType::DagStructureChanged,
            "parallel-block-processed" => WrpcEventType::ParallelBlockProcessed,
            "virtual-chain-merged" => WrpcEventType::VirtualChainMerged,
            "consensus-state-changed" => WrpcEventType::ConsensusStateChanged,
            "network-difficulty-changed" => WrpcEventType::NetworkDifficultyChanged,
            "mempool-state-changed" => WrpcEventType::MempoolStateChanged,
            "peer-connection-changed" => WrpcEventType::PeerConnectionChanged,
            _ => return Err(format!("Unknown event type: {}", event_type).into()),
        };
        
        self.inner.subscribe(event_enum, handler).await
    }
    
    /// Send RPC Call
    pub async fn call(&self, method: &str, request: JsValue) -> Result<JsValue, JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)?;
        let response = self.inner.call_simple(method, request).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    /// Send Notification
    pub async fn notify(&self, method: &str, request: JsValue) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)?;
        self.inner.notify(method, request).await
    }
    
    /// Tondi-specific RPC methods
    
    /// Get subnetwork information
    pub async fn get_subnetwork(&mut self, subnetwork_id: u32, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_subnetwork(subnetwork_id, callback).await
    }
    
    /// Get virtual chain from block
    pub async fn get_virtual_chain_from_block(&mut self, block_hash: &str, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_virtual_chain_from_block(block_hash, callback).await
    }
    
    /// Get block DAG information
    pub async fn get_block_dag_info(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_block_dag_info(callback).await
    }
    
    /// Get sink information
    pub async fn get_sink(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_sink(callback).await
    }
    
    /// Get mempool entries by addresses
    pub async fn get_mempool_entries_by_addresses(&mut self, addresses: JsValue, callback: js_sys::Function) -> Result<(), JsValue> {
        let addresses: Vec<String> = serde_wasm_bindgen::from_value(addresses)?;
        self.inner.get_mempool_entries_by_addresses(addresses, callback).await
    }
    
    /// Get UTXOs by addresses
    pub async fn get_utxos_by_addresses(&mut self, addresses: JsValue, callback: js_sys::Function) -> Result<(), JsValue> {
        let addresses: Vec<String> = serde_wasm_bindgen::from_value(addresses)?;
        self.inner.get_utxos_by_addresses(addresses, callback).await
    }
    
    /// Get balance by address
    pub async fn get_balance_by_address(&mut self, address: &str, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_balance_by_address(address, callback).await
    }
    
    /// Get balances by addresses
    pub async fn get_balances_by_addresses(&mut self, addresses: JsValue, callback: js_sys::Function) -> Result<(), JsValue> {
        let addresses: Vec<String> = serde_wasm_bindgen::from_value(addresses)?;
        self.inner.get_balances_by_addresses(addresses, callback).await
    }
    
    /// Get coin supply
    pub async fn get_coin_supply(&mut self, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.get_coin_supply(callback).await
    }
    
    /// Estimate network hashes per second
    pub async fn estimate_network_hashes_per_second(&mut self, window_size: u32, callback: js_sys::Function) -> Result<(), JsValue> {
        self.inner.estimate_network_hashes_per_second(window_size, callback).await
    }
}
