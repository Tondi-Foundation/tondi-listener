use nill::{Nil, nil};
use tower_http::cors::CorsLayer;
use tondi_scan_h2c::{
    tonic::{codec::CompressionEncoding::Gzip, transport::Server},
    web::GrpcWebLayer,
};
use tondi_scan_h2s::pingpong;
use tondi_scan_lib::log::{info, init_tracing_subscriber_log};
use tondi_scan_server::{
    ctx::Context,
    error::Result,
    middleware,
};

#[tokio::main]
async fn main() -> Result<Nil> {
    // Initialize logging
    init_tracing_subscriber_log();
    
    // Create configuration and context from environment variables
    let ctx = Context::from_env()?;
    
    info!("Server starting...");
    info!("Environment: {}", ctx.config.environment);
    info!("Log level: {}", ctx.log_level());
    info!("Listening address: {}", ctx.config.host_url);
    
    let socket = ctx.config.host_url.parse()?;

    let service = pingpong::service().accept_compressed(Gzip).send_compressed(Gzip);

    // Select middleware based on environment
    let cors_layer = if ctx.is_production() {
        middleware::cors::strict_cors()
    } else {
        middleware::cors::cors(ctx.cors_config())
    };

    Server::builder()
        .accept_http1(true)
        .layer(cors_layer)
        .layer(GrpcWebLayer::new())
        .add_service(service)
        .serve(socket)
        .await?;

    info!("Server stopped");
    Ok(nil)
}
