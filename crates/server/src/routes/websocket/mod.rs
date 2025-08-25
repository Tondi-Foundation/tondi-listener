use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use tondi_scan_lib::log::{error, info, warn};
use tokio::sync::broadcast;

use crate::ctx::Context;

pub async fn handler(client_pool: ClientPool, ws: WebSocketUpgrade) -> Result<Response> {
    let client = client_pool.get().await?;
    let listeners = client.listener_manager.clone();

    let ret = ws.on_upgrade(async |ws| {
        info!("== WebSocketUpgrade ==");
        if let Err(err) = handle_socket(ws, listeners).await {
            error!("Handle Socket Faile: {err}");
        };
    });
    Ok(ret)
}

pub async fn handle_socket(mut socket: WebSocket, listeners: Arc<ListenerManager>) -> Result<Nil> {
    warn!("== Handle Socket ==");
    let rx = listeners.get(&EventType::BlockAdded)?;
    while let Ok(notification) = rx.recv().await {
        info!(?notification, "== Notification ==");
    }

    // while let Some(event) = socket.recv().await {
    //     let ev = match event {
    //         Ok(ev) => ev,
    //         Err(err) => {
    //             warn!("Disconnected: {err}");
    //             return;
    //         },
    //     };

    //     info!("{ev:?}");

    //     if let Err(err) = socket.send(ev).await {
    //         error!("Send Failed: {err}");
    //         return;
    //     }
    // }
    Ok(nil)
}
