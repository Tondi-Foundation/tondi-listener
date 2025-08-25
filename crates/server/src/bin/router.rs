use nill::{Nil, nil};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tondi_listener_library::log::{info, init_tracing_subscriber_log};
use tondi_listener_server::{
    ctx::Context,
    error::Result,
    routes,
};

#[tokio::main]
async fn main() -> Result<Nil> {
    init_tracing_subscriber_log();

    let ctx = Context::from_env()?;
    let socket: SocketAddr = ctx.config.host_url.parse()?;
    info!("Server running: http://{socket}");

    let router = routes::router(ctx).await?;

    let listen = TcpListener::bind(socket).await?;
    axum::serve(listen, router.into_make_service()).await?;
    // .with_graceful_shutdown();

    Ok(nil)
}
