use nill::{Nil, nil};
use tower_http::cors::CorsLayer;
use xscan_h2c::{
    tonic::{codec::CompressionEncoding::Gzip, transport::Server},
    web::GrpcWebLayer,
};
use xscan_h2s::pingpong;
use xscan_lib::log::{info, init_tracing_subscriber_log};
use xscan_server::error::Result;

const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> Result<Nil> {
    init_tracing_subscriber_log();
    info!("Server running");

    let socket = DEFAULT_SOCKET_ADDR.parse()?;

    let service = pingpong::service().accept_compressed(Gzip).send_compressed(Gzip);

    Server::builder()
        .accept_http1(true)
        .layer(CorsLayer::permissive())
        .layer(GrpcWebLayer::new())
        .add_service(service)
        .serve(socket)
        .await?;

    Ok(nil)
}
