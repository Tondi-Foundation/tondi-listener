use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent};
use wasm_bindgen::JsCast;
use log;

/// wRPC Client Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcConfig {
    pub url: String,
    pub encoding: String,
    pub network: String,
}

impl Default for WrpcConfig {
    fn default() -> Self {
        Self {
            url: "ws://8.210.45.192:18610".to_string(),
            encoding: "json".to_string(),
            network: "devnet".to_string(),
        }
    }
}

/// wRPC Event Type Enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrpcEventType {
    BlockAdded,
    VirtualChainChanged,
    FinalityConflict,
    FinalityConflictResolved,
    UtxosChanged,
    SinkBlueScoreChanged,
    VirtualDaaScoreChanged,
    PruningPointUtxoSetOverride,
    NewBlockTemplate,
}

impl WrpcEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WrpcEventType::BlockAdded => "block-added",
            WrpcEventType::VirtualChainChanged => "virtual-chain-changed",
            WrpcEventType::FinalityConflict => "finality-conflict",
            WrpcEventType::FinalityConflictResolved => "finality-conflict-resolved",
            WrpcEventType::UtxosChanged => "utxos-changed",
            WrpcEventType::SinkBlueScoreChanged => "sink-blue-score-changed",
            WrpcEventType::VirtualDaaScoreChanged => "virtual-daa-score-changed",
            WrpcEventType::PruningPointUtxoSetOverride => "pruning-point-utxo-set-override",
            WrpcEventType::NewBlockTemplate => "new-block-template",
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

/// wRPC Client Struct
pub struct WrpcClient {
    websocket: Option<WebSocket>,
    config: WrpcConfig,
    event_handlers: std::collections::HashMap<String, js_sys::Function>,
    connected: bool,
}

impl WrpcClient {
    /// Create new wRPC Client
    pub fn new(config: WrpcConfig) -> Result<Self, JsValue> {
        Ok(Self {
            websocket: None,
            config,
            event_handlers: std::collections::HashMap::new(),
            connected: false,
        })
    }
    
    /// Connect to wRPC Server
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        log::info!("Connecting to wRPC server: {}", self.config.url);
        
        // Create WebSocket Connection
        let websocket = WebSocket::new(&self.config.url)?;
        
        // Set Event Handler
        let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
            if let Some(text) = event.data().dyn_into::<js_sys::JsString>().ok().and_then(|s| s.as_string()) {
                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    log::debug!("Received WebSocket message: {:?}", data);
                    // TODO: Handle Message
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        
        let onopen_callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            log::info!("WebSocket connection opened");
        }) as Box<dyn FnMut(web_sys::Event)>);
        
        let onclose_callback = Closure::wrap(Box::new(move |_event: CloseEvent| {
            log::info!("WebSocket connection closed");
        }) as Box<dyn FnMut(CloseEvent)>);
        
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
    
    /// Send RPC Call
    pub async fn call<Request>(&self, method: &str, request: Request) -> Result<Value, JsValue>
    where
        Request: serde::Serialize + 'static,
    {
        log::debug!("Making RPC call to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let call_msg = serde_json::json!({
                "method": method,
                "params": request,
                "id": js_sys::Date::now() as u64
            });
            
            let msg_str = serde_json::to_string(&call_msg)
                .map_err(|e| format!("Failed to serialize call message: {}", e))?;
            
            websocket.send_with_str(&msg_str)?;
            
            // TODO: Wait for Response
            Ok(serde_json::json!({"status": "sent"}))
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
            _ => return Err(format!("Unknown event type: {}", event_type).into()),
        };
        
        self.inner.subscribe(event_enum, handler).await
    }
    
    /// Send RPC Call
    pub async fn call(&self, method: &str, request: JsValue) -> Result<JsValue, JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)?;
        let response = self.inner.call(method, request).await?;
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    /// Send Notification
    pub async fn notify(&self, method: &str, request: JsValue) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)?;
        self.inner.notify(method, request).await
    }
}
