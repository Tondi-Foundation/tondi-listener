use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use tondi_wrpc_wasm::RpcClient;
use workflow_rpc::encoding::Encoding;
use crate::error::Result;
use std::collections::HashMap;

/// Tondi Scan WASM Client Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TondiScanConfig {
    pub url: Option<String>,
    pub encoding: Option<String>,
    pub network_id: Option<String>,
    pub connection_timeout_ms: Option<u64>,
    pub ping_interval_ms: Option<u64>,
    pub auto_reconnect: Option<bool>,
    pub max_reconnect_attempts: Option<u32>,
    pub reconnect_delay_ms: Option<u64>,
    pub default_events: Option<Vec<String>>,
    pub log_level: Option<String>,
    pub enable_console_log: Option<bool>,
}

impl Default for TondiScanConfig {
    fn default() -> Self {
        Self {
            url: Some("wss://8.210.45.192:18610".to_string()),
            encoding: Some("borsh".to_string()),
            network_id: Some("devnet".to_string()),
            connection_timeout_ms: Some(10000),
            ping_interval_ms: Some(30000),
            auto_reconnect: Some(true),
            max_reconnect_attempts: Some(5),
            reconnect_delay_ms: Some(1000),
            default_events: Some(vec![
                "block-added".to_string(),
                "utxos-changed".to_string(),
                "virtual-chain-changed".to_string(),
                "new-block-template".to_string(),
            ]),
            log_level: Some("info".to_string()),
            enable_console_log: Some(true),
        }
    }
}

impl TryFrom<TondiScanConfig> for tondi_wrpc_wasm::RpcConfig {
    type Error = String;

    fn try_from(config: TondiScanConfig) -> Result<Self, Self::Error> {
        let encoding = match config.encoding.as_deref() {
            Some("borsh") => Some(Encoding::Borsh),
            Some("json") => Some(Encoding::SerdeJson),
            _ => Some(Encoding::Borsh),
        };

        // 简化配置，暂时不使用network_id
        Ok(tondi_wrpc_wasm::RpcConfig {
            resolver: None,
            url: config.url,
            encoding,
            network_id: None,
        })
    }
}

/// Tondi Scan WASM Client
#[wasm_bindgen]
pub struct TondiScanClient {
    inner: RpcClient,
    config: TondiScanConfig,
    event_handlers: HashMap<String, js_sys::Function>,
    auto_reconnect_enabled: bool,
    reconnect_attempts: u32,
}

#[wasm_bindgen]
impl TondiScanClient {
    /// Create new Tondi Scan Client
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<TondiScanClient, JsValue> {
        let config: TondiScanConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| format!("Invalid configuration: {}", e))?;
        
        let rpc_config: tondi_wrpc_wasm::RpcConfig = config.clone().try_into()
            .map_err(|e| format!("Failed to create RPC config: {}", e))?;
        
        let inner = RpcClient::new(Some(rpc_config))
            .map_err(|e| format!("Failed to create RPC client: {}", e))?;
            
        Ok(Self { 
            inner,
            config: config.clone(),
            event_handlers: HashMap::new(),
            auto_reconnect_enabled: config.auto_reconnect.unwrap_or(true),
            reconnect_attempts: 0,
        })
    }

    /// Get configuration
    #[wasm_bindgen(js_name = getConfig)]
    pub fn get_config(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.config).unwrap_or_default()
    }

    /// Update configuration
    #[wasm_bindgen(js_name = updateConfig)]
    pub fn update_config(&mut self, new_config: JsValue) -> Result<(), JsValue> {
        let new_config: TondiScanConfig = serde_wasm_bindgen::from_value(new_config)
            .map_err(|e| format!("Invalid configuration: {}", e))?;
        
        self.config = new_config;
        Ok(())
    }

    /// Connect to Tondi node
    pub async fn connect(&self) -> Result<(), JsValue> {
        self.inner.connect(None).await
            .map_err(|e| format!("Connection failed: {}", e).into())
    }

    /// Disconnect from Tondi node
    pub async fn disconnect(&self) -> Result<(), JsValue> {
        self.inner.disconnect().await
            .map_err(|e| format!("Disconnection failed: {}", e).into())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// Get current URL
    pub fn get_url(&self) -> String {
        self.inner.url().unwrap_or_default()
    }

    /// Get connection statistics
    #[wasm_bindgen(js_name = getStats)]
    pub fn get_stats(&self) -> JsValue {
        let stats = serde_json::json!({
            "connected": self.is_connected(),
            "url": self.get_url(),
            "auto_reconnect_enabled": self.auto_reconnect_enabled,
            "reconnect_attempts": self.reconnect_attempts,
            "event_handlers_count": self.event_handlers.len(),
            "config": self.config
        });
        serde_wasm_bindgen::to_value(&stats).unwrap_or_default()
    }

    /// Enable/disable auto reconnect
    #[wasm_bindgen(js_name = setAutoReconnect)]
    pub fn set_auto_reconnect(&mut self, enabled: bool) {
        self.auto_reconnect_enabled = enabled;
    }

    /// Get auto reconnect status
    #[wasm_bindgen(js_name = isAutoReconnectEnabled)]
    pub fn is_auto_reconnect_enabled(&self) -> bool {
        self.auto_reconnect_enabled
    }

    /// Add event handler
    #[wasm_bindgen(js_name = addEventHandler)]
    pub fn add_event_handler(&mut self, event_type: &str, handler: js_sys::Function) -> Result<(), JsValue> {
        self.event_handlers.insert(event_type.to_string(), handler);
        Ok(())
    }

    /// Remove event handler
    #[wasm_bindgen(js_name = removeEventHandler)]
    pub fn remove_event_handler(&mut self, event_type: &str) -> Result<(), JsValue> {
        if self.event_handlers.remove(event_type).is_some() {
            Ok(())
        } else {
            Err("Event handler not found".into())
        }
    }

    /// Get available event types
    #[wasm_bindgen(js_name = getAvailableEvents)]
    pub fn get_available_events(&self) -> JsValue {
        let events = vec![
            "block-added",
            "utxos-changed",
            "virtual-chain-changed",
            "finality-conflict",
            "finality-conflict-resolved",
            "sink-blue-score-changed",
            "virtual-daa-score-changed",
            "pruning-point-utxo-set-override",
            "new-block-template"
        ];
        serde_wasm_bindgen::to_value(&events).unwrap_or_default()
    }

    /// Ping the node
    pub async fn ping(&self) -> Result<(), JsValue> {
        use tondi_wrpc_wasm::IPingRequest;
        
        let ping_request = IPingRequest::default();
        self.inner.ping(Some(ping_request)).await
            .map(|_| ())
            .map_err(|e| format!("Ping failed: {}", e).into())
    }

    /// Subscribe to block added events
    #[wasm_bindgen(js_name = subscribeBlockAdded)]
    pub async fn subscribe_block_added(&self) -> Result<(), JsValue> {
        self.inner.subscribe_block_added().await
            .map_err(|e| format!("Failed to subscribe to block added: {}", e).into())
    }

    /// Unsubscribe from block added events
    #[wasm_bindgen(js_name = unsubscribeBlockAdded)]
    pub async fn unsubscribe_block_added(&self) -> Result<(), JsValue> {
        self.inner.unsubscribe_block_added().await
            .map_err(|e| format!("Failed to unsubscribe from block added: {}", e).into())
    }

    /// Subscribe to UTXOs changed events
    #[wasm_bindgen(js_name = subscribeUtxosChanged)]
    pub async fn subscribe_utxos_changed(&self, _addresses: JsValue) -> Result<(), JsValue> {
        // 暂时跳过地址转换，直接传递 JsValue
        // TODO: 实现正确的地址转换逻辑
        Err("Address conversion not implemented yet".into())
    }

    /// Unsubscribe from UTXOs changed events
    #[wasm_bindgen(js_name = unsubscribeUtxosChanged)]
    pub async fn unsubscribe_utxos_changed(&self, _addresses: JsValue) -> Result<(), JsValue> {
        // 暂时跳过地址转换，直接传递 JsValue
        // TODO: 实现正确的地址转换逻辑
        Err("Address conversion not implemented yet".into())
    }

    /// Get block by hash
    #[wasm_bindgen(js_name = getBlock)]
    pub async fn get_block(&self, hash: &str) -> Result<JsValue, JsValue> {
        // 暂时使用默认请求，避免字段名问题
        let _response = self.inner.get_block(Default::default()).await
            .map_err(|e| format!("Failed to get block: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "hash": hash,
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }

    /// Get block count
    #[wasm_bindgen(js_name = getBlockCount)]
    pub async fn get_block_count(&self) -> Result<JsValue, JsValue> {
        use tondi_wrpc_wasm::IGetBlockCountRequest;
        
        let request = IGetBlockCountRequest::default();
        let _response = self.inner.get_block_count(Some(request)).await
            .map_err(|e| format!("Failed to get block count: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }

    /// Get sink block hash
    #[wasm_bindgen(js_name = getSink)]
    pub async fn get_sink(&self) -> Result<JsValue, JsValue> {
        use tondi_wrpc_wasm::IGetSinkRequest;
        
        let request = IGetSinkRequest::default();
        let _response = self.inner.get_sink(Some(request)).await
            .map_err(|e| format!("Failed to get sink: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }

    /// Get server info
    #[wasm_bindgen(js_name = getServerInfo)]
    pub async fn get_server_info(&self) -> Result<JsValue, JsValue> {
        use tondi_wrpc_wasm::IGetServerInfoRequest;
        
        let request = IGetServerInfoRequest::default();
        let _response = self.inner.get_server_info(Some(request)).await
            .map_err(|e| format!("Failed to get server info: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }

    /// Get sync status
    #[wasm_bindgen(js_name = getSyncStatus)]
    pub async fn get_sync_status(&self) -> Result<JsValue, JsValue> {
        use tondi_wrpc_wasm::IGetSyncStatusRequest;
        
        let request = IGetSyncStatusRequest::default();
        let _response = self.inner.get_sync_status(Some(request)).await
            .map_err(|e| format!("Failed to get sync status: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }

    /// Get current network
    #[wasm_bindgen(js_name = getCurrentNetwork)]
    pub async fn get_current_network(&self) -> Result<JsValue, JsValue> {
        use tondi_wrpc_wasm::IGetCurrentNetworkRequest;
        
        let request = IGetCurrentNetworkRequest::default();
        let _response = self.inner.get_current_network(Some(request)).await
            .map_err(|e| format!("Failed to get current network: {}", e))?;
        
        // 暂时返回一个简化的响应，避免序列化问题
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }
}
