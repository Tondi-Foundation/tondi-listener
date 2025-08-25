pub mod event;
pub mod state;

use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
};
use nill::{Nil, nil};
use xscan_lib::log::{error, info};

use crate::{
    error::Result,
    routes::websocket::{event::Event, state::State},
};

pub async fn handler(state: State, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(async |ws| {
        info!("== WebSocketUpgrade ==");
        if let Err(err) = handle_socket(ws, state).await {
            error!("Handle Socket Fail: {err}");
        };
    })
}

pub async fn handle_socket(mut socket: WebSocket, mut state: State) -> Result<Nil> {
    info!("== Handle Socket ==");

    let consumer = state.consumer.clone();
    state.rt.spawn(async move {
        while let Ok(notification) = consumer.recv().await {
            let event = Event::from(notification);
            // TODO: Just Send Client Ask
            socket.send(event.try_into()?).await?;
        }
        Ok(nil)
    });

    // TODO: handle Error
    state.rt.join_all().await;

    Ok(nil)
}
