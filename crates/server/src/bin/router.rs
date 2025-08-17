use std::net::SocketAddr;

use nill::{Nil, nil};
use tokio::net::TcpListener;
use xscan_lib::log::{info, init_tracing_subscriber_log};
use xscan_server::{
    ctx::{Context, config::Config},
    error::Result,
    routes::router,
};

#[tokio::main]
async fn main() -> Result<Nil> {
    init_tracing_subscriber_log();

    let config = Config::default();
    let socket: SocketAddr = config.host_url.parse()?;
    info!("Server running: http://{socket}");

    let ctx = Context::new(config)?;
    let router = router(ctx).await?;

    let listen = TcpListener::bind(socket).await?;
    axum::serve(listen, router).await?;
    // .with_graceful_shutdown();

    Ok(nil)
}
