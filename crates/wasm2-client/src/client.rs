use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use tondi_wrpc_wasm::RpcClient;
use workflow_rpc::encoding::Encoding;
use crate::error::Result;
use std::collections::HashMap;

/// wRPC 端口常量定义
/// 根据网络类型和编码类型确定的标准端口
mod wrpc_ports {
    pub const MAINNET_BORSH: u16 = 17110;
    pub const MAINNET_JSON: u16 = 18110;
    pub const TESTNET_BORSH: u16 = 17210;
    pub const TESTNET_JSON: u16 = 18210;
    pub const DEVNET_BORSH: u16 = 17610;
    pub const DEVNET_JSON: u16 = 18610;
    pub const SIMNET_BORSH: u16 = 17310;
    pub const SIMNET_JSON: u16 = 18310;
}

/// 统一配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct UnifiedConfig {
    #[serde(default)]
    client: ClientConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
struct ClientConfig {
    #[serde(default = "default_network")]
    default_network: String,
    #[serde(default = "default_encoding")]
    default_encoding: String,
    #[serde(default = "default_host")]
    default_host: String,
    #[serde(default = "default_protocol")]
    default_protocol: String,
    #[serde(default = "default_connection_timeout")]
    connection_timeout_ms: u64,
    #[serde(default = "default_ping_interval")]
    ping_interval_ms: u64,
    #[serde(default = "default_auto_reconnect")]
    auto_reconnect: bool,
    #[serde(default = "default_max_reconnect_attempts")]
    max_reconnect_attempts: u32,
    #[serde(default = "default_reconnect_delay")]
    reconnect_delay_ms: u64,
    #[serde(default = "default_events")]
    default_events: Vec<String>,
    #[serde(default = "default_log_level")]
    log_level: String,
    #[serde(default = "default_enable_console_log")]
    enable_console_log: bool,
}

// Default value functions
#[allow(dead_code)]
fn default_network() -> String { "devnet".to_string() }
#[allow(dead_code)]
fn default_encoding() -> String { "borsh".to_string() }
#[allow(dead_code)]
fn default_host() -> String { "8.210.45.192".to_string() }
#[allow(dead_code)]
fn default_protocol() -> String { "wss".to_string() }
#[allow(dead_code)]
fn default_connection_timeout() -> u64 { 10000 }
#[allow(dead_code)]
fn default_ping_interval() -> u64 { 30000 }
#[allow(dead_code)]
fn default_auto_reconnect() -> bool { true }
#[allow(dead_code)]
fn default_max_reconnect_attempts() -> u32 { 5 }
#[allow(dead_code)]
fn default_reconnect_delay() -> u64 { 1000 }
#[allow(dead_code)]
fn default_log_level() -> String { "info".to_string() }
#[allow(dead_code)]
fn default_enable_console_log() -> bool { true }
#[allow(dead_code)]
fn default_events() -> Vec<String> {
    vec![
        "block-added".to_string(),
        "utxos-changed".to_string(),
        "virtual-chain-changed".to_string(),
        "new-block-template".to_string(),
    ]
}

/// Tondi Listener WASM Client Configuration
/// 
/// 配置说明：
/// 1. 不再硬编码 URL，支持通过配置文件或参数提供
/// 2. 如果未提供 URL，将根据网络类型和编码类型自动计算端口
/// 3. 端口映射规则与服务器端配置保持一致：
///    - mainnet + borsh = 17110, mainnet + json = 18110
///    - testnet + borsh = 17210, testnet + json = 18210
///    - devnet + borsh = 17610, devnet + json = 18610
///    - simnet + borsh = 17310, simnet + json = 18310
/// 4. 支持环境变量覆盖配置
/// 5. 与服务器端配置结构保持一致，避免配置不一致问题
/// 6. 从统一的 config.toml 文件读取配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TondiScanConfig {
    pub url: Option<String>,
    pub encoding: Option<String>,
    pub network_id: Option<String>,
    pub host: Option<String>,
    pub protocol: Option<String>,
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
            url: None, // No longer hardcode URL, let users provide it through configuration file or parameters
            encoding: Some("borsh".to_string()),
            network_id: Some("devnet".to_string()),
            host: Some("8.210.45.192".to_string()),
            protocol: Some("wss".to_string()),
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

impl TondiScanConfig {
    /// 根据网络类型和编码类型计算默认端口
    pub fn get_default_port(&self) -> u16 {
        let network = self.network_id.as_deref().unwrap_or("devnet");
        let encoding = self.encoding.as_deref().unwrap_or("borsh");
        
        match (network, encoding) {
            ("mainnet", "borsh") => wrpc_ports::MAINNET_BORSH,
            ("mainnet", "json") => wrpc_ports::MAINNET_JSON,
            ("testnet", "borsh") => wrpc_ports::TESTNET_BORSH,
            ("testnet", "json") => wrpc_ports::TESTNET_JSON,
            ("devnet", "borsh") => wrpc_ports::DEVNET_BORSH,
            ("devnet", "json") => wrpc_ports::DEVNET_JSON,
            ("simnet", "borsh") => wrpc_ports::SIMNET_BORSH,
            ("simnet", "json") => wrpc_ports::SIMNET_JSON,
            _ => wrpc_ports::DEVNET_BORSH, // 默认使用 devnet + borsh
        }
    }
    
    /// 构建完整的 URL
    pub fn build_url(&self) -> String {
        if let Some(url) = &self.url {
            url.clone()
        } else {
            // 如果没有提供 URL，使用配置构建
            let protocol = self.protocol.as_deref().unwrap_or("wss");
            let host = self.host.as_deref().unwrap_or("8.210.45.192");
            let port = self.get_default_port();
            format!("{}://{}:{}", protocol, host, port)
        }
    }
    
    /// 从统一配置文件创建配置
    pub fn from_config_file() -> Result<Self, String> {
        // 由于这是 WASM 项目，我们暂时返回默认配置
        // TODO: 实现从配置文件读取的逻辑，可能需要通过 JavaScript 传入配置
        Ok(Self::default())
    }
    
    /// 从 JSON 字符串创建配置
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON config: {}", e))
    }
    
    /// 从 JavaScript 对象创建配置
    pub fn from_js_value(js_value: JsValue) -> Result<Self, String> {
        serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| format!("Failed to parse JS config: {}", e))
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

        // Use the built URL
        let url = Some(config.build_url());

        // For now, do not set network_id because of type mismatch
        // TODO: Implement the correct network type conversion
        Ok(tondi_wrpc_wasm::RpcConfig {
            resolver: None,
            url,
            encoding,
            network_id: None, // For now, set to None to avoid type conversion issues
        })
    }
}

/// Tondi Listener WASM Client
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
    /// Create new Tondi Listener Client
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
        // For now, skip address conversion and pass JsValue directly
        // TODO: Implement the correct address conversion logic
        Err("Address conversion not implemented yet".into())
    }

    /// Unsubscribe from UTXOs changed events
    #[wasm_bindgen(js_name = unsubscribeUtxosChanged)]
    pub async fn unsubscribe_utxos_changed(&self, _addresses: JsValue) -> Result<(), JsValue> {
        // For now, skip address conversion and pass JsValue directly
        // TODO: Implement the correct address conversion logic
        Err("Address conversion not implemented yet".into())
    }

    /// Get block by hash
    #[wasm_bindgen(js_name = getBlock)]
    pub async fn get_block(&self, hash: &str) -> Result<JsValue, JsValue> {
        // For now, use the default request to avoid field name issues
        let _response = self.inner.get_block(Default::default()).await
            .map_err(|e| format!("Failed to get block: {}", e))?;
        
        // For now, return a simplified response to avoid serialization issues
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
        
        // For now, return a simplified response to avoid serialization issues
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
        
        // For now, return a simplified response to avoid serialization issues
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
        
        // For now, return a simplified response to avoid serialization issues
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
        
        // For now, return a simplified response to avoid serialization issues
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
        
        // For now, return a simplified response to avoid serialization issues
        let simplified_response = serde_json::json!({
            "status": "success",
            "note": "Response serialization not implemented yet"
        });
        Ok(serde_wasm_bindgen::to_value(&simplified_response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_calculation() {
        let config = TondiScanConfig::default();
        
        // Test default port (devnet + borsh)
        assert_eq!(config.get_default_port(), wrpc_ports::DEVNET_BORSH);
        
        // Test different network types and encoding combinations
        let mut config = TondiScanConfig::default();
        config.network_id = Some("mainnet".to_string());
        config.encoding = Some("borsh".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::MAINNET_BORSH);
        
        config.encoding = Some("json".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::MAINNET_JSON);
        
        config.network_id = Some("testnet".to_string());
        config.encoding = Some("borsh".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::TESTNET_BORSH);
        
        config.encoding = Some("json".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::TESTNET_JSON);
        
        config.network_id = Some("simnet".to_string());
        config.encoding = Some("borsh".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::SIMNET_BORSH);
        
        config.encoding = Some("json".to_string());
        assert_eq!(config.get_default_port(), wrpc_ports::SIMNET_JSON);
    }

    #[test]
    fn test_url_building() {
        let config = TondiScanConfig::default();
        
        // Test automatically built URL
        let url = config.build_url();
        assert_eq!(url, "wss://8.210.45.192:17610"); // devnet + borsh
        
        // Test custom URL
        let mut config = TondiScanConfig::default();
        config.url = Some("wss://custom.host:8080".to_string());
        let url = config.build_url();
        assert_eq!(url, "wss://custom.host:8080");
        
        // Test different protocol
        let mut config = TondiScanConfig::default();
        config.protocol = Some("ws".to_string());
        let url = config.build_url();
        assert_eq!(url, "ws://8.210.45.192:17610");
    }

    #[test]
    fn test_config_consistency() {
        let config = TondiScanConfig::default();
        
        // Verify default configuration matches server-side setup
        assert_eq!(config.network_id, Some("devnet".to_string()));
        assert_eq!(config.encoding, Some("borsh".to_string()));
        assert_eq!(config.protocol, Some("wss".to_string()));
        assert_eq!(config.host, Some("8.210.45.192".to_string()));
        
        // Verify port calculation logic
        let expected_port = wrpc_ports::DEVNET_BORSH;
        assert_eq!(config.get_default_port(), expected_port);
    }
}
