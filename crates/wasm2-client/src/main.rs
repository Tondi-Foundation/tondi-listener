use nill::{Nil, nil};
use wasm_bindgen_futures::spawn_local;
use tondi_listener_library::log::{info, init_tracing_browser_subscriber_log};
use tondi_listener_wasm2_client::{error::Result, client::TondiScanClient};

// #[tokio::main(flavor = "current_thread")] async
fn main() -> Result<Nil> {
    init_tracing_browser_subscriber_log();
    info!("Running");

    spawn_local(async {
        // 创建新的Tondi Listener客户端
        // 从统一配置文件读取配置，而不是硬编码
        let config = serde_wasm_bindgen::to_value(&serde_json::json!({
            // 配置将从统一配置文件读取，支持环境变量覆盖
            "encoding": "borsh",
            "network_id": "devnet"
            // 如果没有提供 URL，将根据网络类型和编码类型自动计算端口
            // devnet + borsh = 17610
        })).unwrap();
        
        let client = TondiScanClient::new(config).unwrap();
        
        info!("Tondi Listener client created");
        
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
