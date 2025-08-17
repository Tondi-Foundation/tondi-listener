pub mod chain;
pub mod grpc;
pub mod transaction;
pub mod websocket;

use axum::{Router, response::Html, routing::{get,post,any}};

use crate::{ctx::Context, error::Result, extensions::client_pool, middleware::middleware};

pub async fn index() -> Html<&'static str> {
    Html("Axum Serve")
}

// TODO: Route trait
pub async fn router(ctx: Context) -> Result<Router> {
    let Context { config, .. } = &ctx;
    let client_pool = client_pool::extension(&config.grpc_url).await?;

    let router = Router::new()
        .route("/", get(index))
        .route("/chain/last", get(chain::last::get))
        .route("/transaction/last", get(transaction::last::get))
        .route("/transaction/{id}", get(transaction::_id_::get))
        .route("/grpc", post(grpc::post))
        .route("/websocket", any(websocket::handler))
        .with_state(ctx)
        .layer(client_pool)
        .layer(middleware());

    Ok(router)
}
