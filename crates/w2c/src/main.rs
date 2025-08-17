use nill::{Nil, nil};
use wasm_bindgen_futures::spawn_local;
use xscan_h2c::protowire::Ping;
use xscan_lib::log::{info, init_tracing_browser_subscriber_log};
use xscan_w2c::{error::Result, route::pingpong::pingpong};

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
