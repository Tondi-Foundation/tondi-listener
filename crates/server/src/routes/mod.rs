pub mod chain;
pub mod grpc;
pub mod transaction;
pub mod websocket;

use axum::{Router, response::Html, routing::{get,post}};

use crate::{ctx::Context, error::Result, extensions::client_pool};
use tondi_scan_library::log::info;

pub async fn index() -> Html<&'static str> {
    Html("Axum Serve")
}

// TODO: Route trait
pub async fn router(ctx: Context) -> Result<Router> {
    let Context { config, .. } = &ctx;
    
    // Parse configured event types
    let event_types = config.events.parse_event_types()
        .map_err(|e| crate::error::Error::InternalServerError(format!("Invalid event config: {}", e)))?;
    
    // Select URL and protocol based on configuration
    let (rpc_url, protocol_type) = if config.wrpc.enabled {
        let url = config.wrpc.build_url();
        (url, "wRPC")
    } else {
        (config.grpc_url.clone(), "gRPC")
    };
    
    // Log selected protocol
    info!("Using {} protocol with URL: {}", protocol_type, rpc_url);
    
    // Create client pool with configured events
    let client_pool = client_pool::extension_with_events(
        &rpc_url, 
        &event_types.into_iter().collect::<Vec<_>>()
    ).await?;

    let router = Router::new()
        .route("/", get(index))
        .route("/chain/last", get(chain::last::get))
        .route("/transaction/last", get(transaction::last::get))
        .route("/transaction/{id}", get(transaction::_id_::get))
        .route("/grpc", post(grpc::post))
        .route("/websocket", get(websocket::handler))
        .with_state(client_pool)
        .layer(
            tower::ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(crate::middleware::trace::trace())
                .layer(crate::middleware::cors::cors(&ctx.config.cors))
        );

    Ok(router)
}
