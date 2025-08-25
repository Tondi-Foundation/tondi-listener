use axum::Router;
use nill::{Nil, nil};
use tokio::net::TcpListener;
use tondi_scan_library::log::{info, init_tracing_subscriber_log};
use tondi_scan_server::{
    ctx::Context,
    error::Result,
    middleware,
    routes::{chain, transaction, websocket},
};

#[tokio::main]
async fn main() -> Result<Nil> {
    init_tracing_subscriber_log();

    let config = Context::default().config;
    let socket: SocketAddr = config.host_url.parse()?;
    info!("Server running: http://{socket}");

    let ctx = Context::new(config)?;
    let router = Router::new()
        .merge(chain::router(ctx).await?)
        .merge(transaction::router(ctx).await?)
        .merge(websocket::router(ctx).await?);

    let listen = TcpListener::bind(socket).await?;
    axum::serve(listen, router).await?;
    // .with_graceful_shutdown();

    Ok(nil)
}
