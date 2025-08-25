pub mod cors;
pub mod trace;

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::middleware::{cors::cors, trace::trace};

/// Create middleware stack for the application
pub fn create_middleware_stack() -> impl tower::Layer<axum::routing::Route> + Clone + Send + Sync + 'static {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(trace())
        .layer(cors(&crate::ctx::config::CorsConfig::default()))
        .into_inner()
}
