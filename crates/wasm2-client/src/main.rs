use nill::{Nil, nil};
use wasm_bindgen_futures::spawn_local;
use tondi_scan_library::log::{info, init_tracing_browser_subscriber_log};
use tondi_scan_wasm2_client::{error::Result, client::TondiScanClient};

// #[tokio::main(flavor = "current_thread")] async
fn main() -> Result<Nil> {
    init_tracing_browser_subscriber_log();
    info!("Running");

    spawn_local(async {
        // 创建新的Tondi Scan客户端
        // 不再硬编码 URL，使用配置文件的默认值或让用户通过参数提供
        let config = serde_wasm_bindgen::to_value(&serde_json::json!({
            // "url": "wss://8.210.45.192:18610",  // 已移除硬编码
            "encoding": "borsh",
            "network_id": "devnet"
            // 如果没有提供 URL，将根据网络类型和编码类型自动计算端口
            // devnet + borsh = 17610
        })).unwrap();
        
        let client = TondiScanClient::new(config).unwrap();
        
        info!("Tondi Scan client created");
        
        // 连接测试
        if let Err(e) = client.connect().await {
            info!("Connection failed: {:?}", e);
        } else {
            info!("Connected successfully");
            
            // 测试ping
            if let Err(e) = client.ping().await {
                info!("Ping failed: {:?}", e);
            } else {
                info!("Ping successful");
            }
            
            // 断开连接
            if let Err(e) = client.disconnect().await {
                info!("Disconnect failed: {:?}", e);
            } else {
                info!("Disconnected successfully");
            }
        }
    });

    Ok(nil)
}
