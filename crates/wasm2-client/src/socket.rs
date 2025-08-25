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
    #[error("Configuration error: {0}")]
    Config(String),
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

/// Format error message for JavaScript binding
fn format_js_error(operation: &str, error: &WrpcError) -> String {
    format!("{} failed: {}", operation, error)
}

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
            return Err(WrpcError::Config("URL cannot be empty".to_string()));
        }
        
        if !self.url.starts_with("ws://") && !self.url.starts_with("wss://") {
            return Err(WrpcError::Config("URL must start with ws:// or wss://".to_string()));
        }
        
        if self.reconnect_attempts == 0 {
            return Err(WrpcError::Config("Reconnect attempts must be greater than 0".to_string()));
        }
        
        if self.reconnect_delay_ms == 0 {
            return Err(WrpcError::Config("Reconnect delay must be greater than 0".to_string()));
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
    /// Parse event type string to enum
    pub fn from_str(event_type: &str) -> WrpcResult<Self> {
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
    
    /// Get all event type strings as a vector
    pub fn all_event_strings() -> &'static [&'static str] {
        &[
            "block-added",
            "virtual-chain-changed",
            "finality-conflict",
            "finality-conflict-resolved",
            "utxos-changed",
            "sink-blue-score-changed",
            "virtual-daa-score-changed",
            "pruning-point-utxo-set-override",
            "new-block-template",
        ]
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
    
    /// Get connection statistics
    pub fn get_stats(&self) -> (bool, usize, usize, bool, u32, u32) {
        (
            self.connected,
            self.event_handlers.len(),
            self.pending_requests.len(),
            self.current_reconnect_attempt > 0,
            self.current_reconnect_attempt,
            self.config.reconnect_attempts,
        )
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
        let event_enum = WrpcEventType::from_str(event_type)?;
        
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
        
        // Wait before attempting reconnection (non-blocking in WASM)
        // Note: In WASM environment, this is effectively a no-op
        // but provides the intended delay behavior
        // TODO: Implement proper async delay when gloo_timers is available
        // For now, we'll just proceed without delay to avoid blocking
        
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
            .map_err(|e| format_js_error("Connection", &e).into())
    }
    
    /// Disconnect from Server
    pub async fn disconnect(&mut self) -> Result<(), JsValue> {
        self.inner.disconnect().await
            .map_err(|e| format_js_error("Disconnection", &e).into())
    }
    
    /// Check Connection Status
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
    
    /// Get connection statistics
    pub fn get_stats(&self) -> JsValue {
        let (connected, event_handlers, pending_requests, reconnecting, current_attempt, max_attempts) = self.inner.get_stats();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "connected": connected,
            "event_handlers": event_handlers,
            "pending_requests": pending_requests,
            "reconnecting": reconnecting,
            "reconnect_attempts": current_attempt,
            "max_reconnect_attempts": max_attempts,
        })).unwrap_or_else(|_| {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "error": "Failed to serialize statistics"
            })).unwrap_or_default()
        })
    }
    
    /// Subscribe to Event
    pub async fn subscribe(&mut self, event_type: &str, handler: js_sys::Function) -> Result<(), JsValue> {
        self.inner.subscribe(event_type, handler).await
            .map_err(|e| format_js_error("Subscription", &e).into())
    }
    
    /// Unsubscribe from Event
    pub async fn unsubscribe(&mut self, event_type: &str) -> Result<(), JsValue> {
        self.inner.unsubscribe(event_type).await
            .map_err(|e| format_js_error("Unsubscription", &e).into())
    }
    
    /// Send RPC Call
    pub async fn call(&self, method: &str, request: JsValue) -> Result<JsValue, JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        let response = self.inner.call_simple(method, request).await
            .map_err(|e| JsValue::from_str(&format_js_error("RPC call", &e)))?;
            
        Ok(serde_wasm_bindgen::to_value(&response)?)
    }
    
    /// Send RPC Call with Callback
    pub async fn call_with_callback(&mut self, method: &str, request: JsValue, callback: js_sys::Function) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        self.inner.call(method, request, callback).await
            .map_err(|e| format_js_error("RPC call", &e).into())
    }
    
    /// Send Notification
    pub async fn notify(&self, method: &str, request: JsValue) -> Result<(), JsValue> {
        let request: Value = serde_wasm_bindgen::from_value(request)
            .map_err(|e| format!("Invalid request: {}", e))?;
            
        self.inner.notify(method, request).await
            .map_err(|e| format_js_error("Notification", &e).into())
    }
    
    /// Attempt to reconnect
    pub async fn reconnect(&mut self) -> Result<(), JsValue> {
        self.inner.reconnect().await
            .map_err(|e| format_js_error("Reconnection", &e).into())
    }
    
    /// Reset reconnection attempts
    pub fn reset_reconnect_attempts(&mut self) {
        self.inner.reset_reconnect_attempts();
    }
    
    /// Get reconnection statistics
    pub fn get_reconnection_stats(&self) -> JsValue {
        let (_, _, _, _, current_attempt, max_attempts) = self.inner.get_stats();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "current_attempt": current_attempt,
            "max_attempts": max_attempts,
        })).unwrap_or_else(|_| {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "error": "Failed to serialize reconnection statistics"
            })).unwrap_or_default()
        })
    }
    
    /// Get available event types
    pub fn get_available_events(&self) -> JsValue {
        let events = WrpcEventType::all_event_strings();
        serde_wasm_bindgen::to_value(&events).unwrap_or_default()
    }
    
    /// Get client configuration
    pub fn get_config(&self) -> JsValue {
        let config = self.inner.config();
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "url": config.url,
            "encoding": config.encoding,
            "network": config.network,
            "reconnect_attempts": config.reconnect_attempts,
            "reconnect_delay_ms": config.reconnect_delay_ms,
        })).unwrap_or_else(|_| {
            serde_wasm_bindgen::to_value(&serde_json::json!({
                "error": "Failed to serialize configuration"
            })).unwrap_or_default()
        })
    }
    
    /// Clear all pending requests
    pub fn clear_pending_requests(&mut self) {
        self.inner.clear_pending_requests();
    }
    
    /// Check if client is currently reconnecting
    pub fn is_reconnecting(&self) -> bool {
        let (_, _, _, reconnecting, _, _) = self.inner.get_stats();
        reconnecting
    }
    
    /// Get current reconnection attempt number
    pub fn get_current_reconnect_attempt(&self) -> u32 {
        let (_, _, _, _, current_attempt, _) = self.inner.get_stats();
        current_attempt
    }
    
    /// Get maximum reconnection attempts
    pub fn get_max_reconnect_attempts(&self) -> u32 {
        let (_, _, _, _, _, max_attempts) = self.inner.get_stats();
        max_attempts
    }
    
    /// Get number of registered event handlers
    pub fn get_event_handler_count(&self) -> usize {
        let (_, event_handlers, _, _, _, _) = self.inner.get_stats();
        event_handlers
    }
    
    /// Get number of pending requests
    pub fn get_pending_request_count(&self) -> usize {
        let (_, _, pending_requests, _, _, _) = self.inner.get_stats();
        pending_requests
    }
    
    /// Validate event type string
    pub fn validate_event_type(&self, event_type: &str) -> bool {
        WrpcEventType::from_str(event_type).is_ok()
    }
    
    /// Get client version information
    pub fn get_version(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&serde_json::json!({
            "name": "tondi-scan-wasm2-client",
            "version": env!("CARGO_PKG_VERSION"),
            "features": ["websocket", "events", "rpc", "reconnection"]
        })).unwrap_or_default()
    }
}
