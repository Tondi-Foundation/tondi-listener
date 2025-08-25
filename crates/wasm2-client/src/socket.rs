use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use web_sys::{WebSocket, MessageEvent, ErrorEvent};
use wasm_bindgen::JsCast;
use log;
use std::collections::HashMap;
use thiserror::Error;

/// Custom error type for wRPC operations
#[derive(Error, Debug, Clone)]
pub enum WrpcError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Invalid event type: {0}")]
    InvalidEventType(String),
    #[error("Max reconnection attempts reached")]
    MaxReconnectAttempts,
}

impl From<JsValue> for WrpcError {
    fn from(js_value: JsValue) -> Self {
        WrpcError::WebSocket(format!("JavaScript error: {:?}", js_value))
    }
}

impl From<serde_json::Error> for WrpcError {
    fn from(err: serde_json::Error) -> Self {
        WrpcError::Serialization(err.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for WrpcError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        WrpcError::Serialization(err.to_string())
    }
}

/// Result type for wRPC operations
pub type WrpcResult<T> = Result<T, WrpcError>;

/// wRPC Client Config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcConfig {
    pub url: String,
    pub encoding: String,
    pub network: String,
    pub reconnect_attempts: u32,
    pub reconnect_delay_ms: u32,
}

impl WrpcConfig {
    /// Validate configuration
    pub fn validate(&self) -> WrpcResult<()> {
        if self.url.is_empty() {
            return Err(WrpcError::Connection("URL cannot be empty".to_string()));
        }
        
        if !self.url.starts_with("ws://") && !self.url.starts_with("wss://") {
            return Err(WrpcError::Connection("URL must start with ws:// or wss://".to_string()));
        }
        
        if self.reconnect_attempts == 0 {
            return Err(WrpcError::Connection("Reconnect attempts must be greater than 0".to_string()));
        }
        
        if self.reconnect_delay_ms == 0 {
            return Err(WrpcError::Connection("Reconnect delay must be greater than 0".to_string()));
        }
        
        Ok(())
    }
    
    /// Create a new config with validation
    pub fn new(url: String, encoding: String, network: String, reconnect_attempts: u32, reconnect_delay_ms: u32) -> WrpcResult<Self> {
        let config = Self {
            url,
            encoding,
            network,
            reconnect_attempts,
            reconnect_delay_ms,
        };
        config.validate()?;
        Ok(config)
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
}

impl WrpcEventType {
    /// Convert event type to string representation
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
    
    /// Check if this is a core blockchain event
    pub fn is_core_event(&self) -> bool {
        true // All events are core events now
    }
    
    /// Get all event types as a vector
    pub fn all_events() -> Vec<Self> {
        vec![
            WrpcEventType::BlockAdded,
            WrpcEventType::VirtualChainChanged,
            WrpcEventType::FinalityConflict,
            WrpcEventType::FinalityConflictResolved,
            WrpcEventType::UtxosChanged,
            WrpcEventType::SinkBlueScoreChanged,
            WrpcEventType::VirtualDaaScoreChanged,
            WrpcEventType::PruningPointUtxoSetOverride,
            WrpcEventType::NewBlockTemplate,
        ]
    }
}

/// wRPC Event Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcEvent {
    pub event_type: WrpcEventType,
    pub data: Value,
    pub timestamp: u64,
}

impl WrpcEvent {
    /// Create a new event with current timestamp
    pub fn new(event_type: WrpcEventType, data: Value) -> Self {
        Self {
            event_type,
            data,
            timestamp: js_sys::Date::now() as u64,
        }
    }
    
    /// Check if this is a core blockchain event
    pub fn is_core_event(&self) -> bool {
        self.event_type.is_core_event()
    }
    
    /// Check if this is a Tondi-specific event
    pub fn is_tondi_event(&self) -> bool {
        false // No Tondi events
    }
}

/// wRPC Response Struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrpcResponse {
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

impl WrpcResponse {
    /// Create a new success response
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }
    
    /// Create a new error response
    pub fn error(id: u64, error: Value) -> Self {
        Self {
            id: Some(id),
            result: None,
            error: Some(error),
        }
    }
    
    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        self.error.is_none() && self.result.is_some()
    }
    
    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
    
    /// Get the result value, returning an error if this is not a success response
    pub fn get_result(&self) -> WrpcResult<&Value> {
        if let Some(result) = &self.result {
            Ok(result)
        } else {
            Err(WrpcError::Rpc("No result in response".to_string()))
        }
    }
    
    /// Get the error value, returning an error if this is not an error response
    pub fn get_error(&self) -> WrpcResult<&Value> {
        if let Some(error) = &self.error {
            Ok(error)
        } else {
            Err(WrpcError::Rpc("No error in response".to_string()))
        }
    }
}

/// wRPC Client Struct
pub struct WrpcClient {
    websocket: Option<WebSocket>,
    config: WrpcConfig,
    event_handlers: HashMap<String, js_sys::Function>,
    pending_requests: HashMap<u64, js_sys::Function>,
    connected: bool,
    current_reconnect_attempt: u32,
}

impl WrpcClient {
    /// Create new wRPC Client
    pub fn new(config: WrpcConfig) -> WrpcResult<Self> {
        // Validate configuration
        config.validate()?;
        
        Ok(Self {
            websocket: None,
            config,
            event_handlers: HashMap::new(),
            pending_requests: HashMap::new(),
            connected: false,
            current_reconnect_attempt: 0,
        })
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &WrpcConfig {
        &self.config
    }
    
    /// Get the number of registered event handlers
    pub fn event_handler_count(&self) -> usize {
        self.event_handlers.len()
    }
    
    /// Get the number of pending requests
    pub fn pending_request_count(&self) -> usize {
        self.pending_requests.len()
    }
    
    /// Check if the client is currently reconnecting
    pub fn is_reconnecting(&self) -> bool {
        self.current_reconnect_attempt > 0
    }
    
    /// Get the current reconnection attempt number
    pub fn current_reconnect_attempt(&self) -> u32 {
        self.current_reconnect_attempt
    }
    
    /// Get the maximum number of reconnection attempts
    pub fn max_reconnect_attempts(&self) -> u32 {
        self.config.reconnect_attempts
    }
    
    /// Connect to wRPC Server
    pub async fn connect(&mut self) -> WrpcResult<()> {
        if self.connected {
            return Err(WrpcError::Connection("Already connected".to_string()));
        }
        
        log::info!("Connecting to wRPC server: {}", self.config.url);
        
        // Create WebSocket Connection
        let websocket = WebSocket::new(&self.config.url)
            .map_err(|e| WrpcError::Connection(format!("Failed to create WebSocket: {:?}", e)))?;
        
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
                            if let Err(e) = handler.call1(&wasm_bindgen::JsValue::NULL, &serde_wasm_bindgen::to_value(&data).unwrap_or_default()) {
                                log::error!("Failed to call event handler for {}: {:?}", method, e);
                            }
                        }
                    } else if let Some(id) = data.get("id").and_then(|i| i.as_u64()) {
                        // This is a response to an RPC call
                        if let Some(callback) = pending_requests.get(&id) {
                            if let Err(e) = callback.call1(&wasm_bindgen::JsValue::NULL, &serde_wasm_bindgen::to_value(&data).unwrap_or_default()) {
                                log::error!("Failed to call RPC callback for id {}: {:?}", id, e);
                            }
                        }
                    }
                } else {
                    log::warn!("Failed to parse WebSocket message as JSON: {}", text);
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
    pub async fn disconnect(&mut self) -> WrpcResult<()> {
        if !self.connected {
            return Err(WrpcError::Connection("Not connected".to_string()));
        }
        
        log::info!("Disconnecting from wRPC server");
        
        if let Some(websocket) = &self.websocket {
            websocket.close()
                .map_err(|e| WrpcError::Connection(format!("Failed to close WebSocket: {:?}", e)))?;
        }
        
        self.websocket = None;
        self.connected = false;
        self.current_reconnect_attempt = 0;
        
        // Clear pending requests since we're disconnecting
        self.clear_pending_requests();
        
        log::info!("Successfully disconnected from wRPC server");
        Ok(())
    }
    
    /// Check Connection Status
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Subscribe to Event
    pub async fn subscribe(&mut self, event_type: &str, handler: js_sys::Function) -> WrpcResult<()> {
        if !self.connected {
            return Err(WrpcError::Connection("Not connected".to_string()));
        }
        
        // Validate event type
        let event_enum = self.parse_event_type(event_type)?;
        
        // Store the handler
        self.event_handlers.insert(event_type.to_string(), handler);
        
        log::debug!("Subscribed to event: {} ({:?})", event_type, event_enum);
        Ok(())
    }
    
    /// Unsubscribe from Event
    pub async fn unsubscribe(&mut self, event_type: &str) -> WrpcResult<()> {
        if self.event_handlers.remove(event_type).is_some() {
            log::debug!("Unsubscribed from event: {}", event_type);
            Ok(())
        } else {
            Err(WrpcError::InvalidEventType(format!("Not subscribed to event: {}", event_type)))
        }
    }
    
    /// Parse event type string to enum
    fn parse_event_type(&self, event_type: &str) -> WrpcResult<WrpcEventType> {
        match event_type {
            "block-added" => Ok(WrpcEventType::BlockAdded),
            "virtual-chain-changed" => Ok(WrpcEventType::VirtualChainChanged),
            "finality-conflict" => Ok(WrpcEventType::FinalityConflict),
            "finality-conflict-resolved" => Ok(WrpcEventType::FinalityConflictResolved),
            "utxos-changed" => Ok(WrpcEventType::UtxosChanged),
            "sink-blue-score-changed" => Ok(WrpcEventType::SinkBlueScoreChanged),
            "virtual-daa-score-changed" => Ok(WrpcEventType::VirtualDaaScoreChanged),
            "pruning-point-utxo-set-override" => Ok(WrpcEventType::PruningPointUtxoSetOverride),
            "new-block-template" => Ok(WrpcEventType::NewBlockTemplate),
            _ => Err(WrpcError::InvalidEventType(format!("Unknown event type: {}", event_type))),
        }
    }
    
    /// Send RPC Call with Response Handling
    pub async fn call<Request>(&mut self, method: &str, request: Request, callback: js_sys::Function) -> WrpcResult<()>
    where
        Request: serde::Serialize + 'static,
    {
        if !self.connected {
            return Err(WrpcError::Connection("Not connected".to_string()));
        }
        
        if method.is_empty() {
            return Err(WrpcError::Rpc("Method name cannot be empty".to_string()));
        }
        
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
                .map_err(|e| WrpcError::Serialization(format!("Failed to serialize call message: {}", e)))?;
            
            websocket.send_with_str(&msg_str)
                .map_err(|e| WrpcError::WebSocket(format!("Failed to send RPC call: {:?}", e)))?;
                
            log::debug!("RPC call sent successfully with ID: {}", request_id);
            Ok(())
        } else {
            Err(WrpcError::Connection("WebSocket not connected".to_string()))
        }
    }
    
    /// Send RPC Call (Legacy method for backward compatibility)
    pub async fn call_simple<Request>(&self, method: &str, request: Request) -> WrpcResult<Value>
    where
        Request: serde::Serialize + 'static,
    {
        if !self.connected {
            return Err(WrpcError::Connection("Not connected".to_string()));
        }
        
        if method.is_empty() {
            return Err(WrpcError::Rpc("Method name cannot be empty".to_string()));
        }
        
        log::debug!("Making simple RPC call to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let call_msg = serde_json::json!({
                "method": method,
                "params": request,
                "id": js_sys::Date::now() as u64
            });
            
            let msg_str = serde_json::to_string(&call_msg)
                .map_err(|e| WrpcError::Serialization(format!("Failed to serialize call message: {}", e)))?;
            
            websocket.send_with_str(&msg_str)
                .map_err(|e| WrpcError::WebSocket(format!("Failed to send RPC call: {:?}", e)))?;
            
            // Return a placeholder response
            Ok(serde_json::json!({
                "status": "sent", 
                "note": "Use call() method for response handling"
            }))
        } else {
            Err(WrpcError::Connection("WebSocket not connected".to_string()))
        }
    }
    
    /// Send Notification
    pub async fn notify<Request>(&self, method: &str, request: Request) -> WrpcResult<()>
    where
        Request: serde::Serialize + 'static,
    {
        if !self.connected {
            return Err(WrpcError::Connection("Not connected".to_string()));
        }
        
        if method.is_empty() {
            return Err(WrpcError::Rpc("Method name cannot be empty".to_string()));
        }
        
        log::debug!("Sending notification to method: {}", method);
        
        if let Some(websocket) = &self.websocket {
            let notify_msg = serde_json::json!({
                "method": method,
                "params": request
            });
            
            let msg_str = serde_json::to_string(&notify_msg)
                .map_err(|e| WrpcError::Serialization(format!("Failed to serialize notification message: {}", e)))?;
            
            websocket.send_with_str(&msg_str)
                .map_err(|e| WrpcError::WebSocket(format!("Failed to send notification: {:?}", e)))?;
                
            log::debug!("Notification sent successfully to method: {}", method);
            Ok(())
        } else {
            Err(WrpcError::Connection("WebSocket not connected".to_string()))
        }
    }
    
    /// Attempt to reconnect
    pub async fn reconnect(&mut self) -> WrpcResult<()> {
        if self.current_reconnect_attempt >= self.config.reconnect_attempts {
            return Err(WrpcError::MaxReconnectAttempts);
        }
        
        if self.connected {
            return Err(WrpcError::Connection("Already connected".to_string()));
        }
        
        self.current_reconnect_attempt += 1;
        log::info!("Attempting to reconnect (attempt {}/{})", self.current_reconnect_attempt, self.config.reconnect_attempts);
        
        // Wait before attempting reconnection
        let delay = std::time::Duration::from_millis(self.config.reconnect_delay_ms as u64);
        std::thread::sleep(delay);
        
        // Attempt to connect
        match self.connect().await {
            Ok(()) => {
                log::info!("Reconnection successful on attempt {}", self.current_reconnect_attempt);
                Ok(())
            }
            Err(e) => {
                log::warn!("Reconnection attempt {} failed: {:?}", self.current_reconnect_attempt, e);
                Err(e)
            }
        }
    }
    
    /// Reset reconnection attempts counter
    pub fn reset_reconnect_attempts(&mut self) {
        self.current_reconnect_attempt = 0;
        log::debug!("Reconnection attempts counter reset");
    }
    
    /// Get reconnection statistics
    pub fn get_reconnection_stats(&self) -> (u32, u32) {
        (self.current_reconnect_attempt, self.config.reconnect_attempts)
    }
    
    /// Clear pending requests
    pub fn clear_pending_requests(&mut self) {
        self.pending_requests.clear();
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
        let config: WrpcConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| format!("Invalid configuration: {}", e))?;
        
        let inner = WrpcClient::new(config)
            .map_err(|e| format!("Failed to create client: {}", e))?;
            
        Ok(Self { inner })
    }
    
    /// Connect to Server
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        self.inner.connect().await
            .map_err(|e| format!("Connection failed: {}", e).into())
    }
    
    /// Disconnect from Server
    pub async fn disconnect(&mut self) -> Result<(), JsValue> {
        self.inner.disconnect().await
            .map_err(|e| format!("Disconnection failed: {}", e).into())
    }
    
    /// Check Connection Status
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    /// Get connection statistics
    pub fn get_stats(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "connected": self.inner.is_connected(),
            "event_handlers": self.inner.event_handler_count(),
            "pending_requests": self.inner.pending_request_count(),
            "reconnecting": self.inner.is_reconnecting(),
            "reconnect_attempts": self.inner.current_reconnect_attempt(),
            "max_reconnect_attempts": self.inner.max_reconnect_attempts(),
        })).unwrap_or_default()
    }
    
    /// Subscribe to Event
    pub async fn subscribe(&mut self, event_type: &str, handler: js_sys::Function) -> Result<(), JsValue> {
        self.inner.subscribe(event_type, handler).await
            .map_err(|e| format!("Subscription failed: {}", e).into())
    }
    
    /// Unsubscribe from Event
    pub async fn unsubscribe(&mut self, event_type: &str) -> Result<(), JsValue> {
        self.inner.unsubscribe(event_type).await
            .map_err(|e| format!("Unsubscription failed: {}", e).into())
    }
    
    /// Send RPC Call
    pub async fn call(&self, method: &str, request: JsValue) -> Result<JsValue, JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        let response = self.inner.call_simple(method, request).await
            .map_err(|e| format!("RPC call failed: {}", e))?;
            
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    /// Send RPC Call with Callback
    pub async fn call_with_callback(&mut self, method: &str, request: JsValue, callback: js_sys::Function) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        self.inner.call(method, request, callback).await
            .map_err(|e| format!("RPC call failed: {}", e).into())
    }
    
    /// Send Notification
    pub async fn notify(&self, method: &str, request: JsValue) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        self.inner.notify(method, request).await
            .map_err(|e| format!("Notification failed: {}", e).into())
    }
    
    /// Attempt to reconnect
    pub async fn reconnect(&mut self) -> Result<(), JsValue> {
        self.inner.reconnect().await
            .map_err(|e| format!("Reconnection failed: {}", e).into())
    }
    
    /// Reset reconnection attempts
    pub fn reset_reconnect_attempts(&mut self) {
        self.inner.reset_reconnect_attempts();
    }
    
    /// Get reconnection statistics
    pub fn get_reconnection_stats(&self) -> JsValue {
        let (current, max) = self.inner.get_reconnection_stats();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "current_attempt": current,
            "max_attempts": max,
        })).unwrap_or_default()
    }
}
