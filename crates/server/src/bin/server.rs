use nill::{Nil, nil};
use tondi_listener_http2_client::{
    tonic::{codec::CompressionEncoding::Gzip, transport::Server},
    web::GrpcWebLayer,
};
use tondi_listener_http2_server::pingpong;
use tondi_listener_library::log::{info, init_tracing_subscriber_log};
use tondi_listener_server::{
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

    let server = Server::builder()
        .accept_http1(true)
        .layer(cors_layer)
        .layer(GrpcWebLayer::new());

    // Use the service directly
    server.serve(socket, service).await?;

    info!("Server stopped");
    Ok(nil)
}
