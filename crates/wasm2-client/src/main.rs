use nill::{Nil, nil};
use wasm_bindgen_futures::spawn_local;
use tondi_scan_http2_client::protowire::Ping;
use tondi_scan_library::log::{info, init_tracing_browser_subscriber_log};
use tondi_scan_wasm2_client::{error::Result, route::pingpong::pingpong};

// #[tokio::main(flavor = "current_thread")] async
fn main() -> Result<Nil> {
    init_tracing_browser_subscriber_log();
    info!("Running");

    spawn_local(async {
        let ping = Ping::default();
        info!(?ping);
        let pong = pingpong(ping).await.unwrap();
        info!(?pong);
    });

    Ok(nil)
}
