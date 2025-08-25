use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum::extract::ws::{Message, WebSocket};
use serde_json::json;

use crate::{
    error::Result,
    extensions::client_pool::ClientPool,
};

pub fn router() -> Router<ClientPool> {
    Router::new().route("/ws", get(handler))
}

pub async fn handler(
    State(_client_pool): State<ClientPool>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        if let Err(e) = handle_socket(socket, _client_pool).await {
            eprintln!("WebSocket error: {}", e);
        }
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    _client_pool: ClientPool,
) -> Result<()> {
    // Send welcome message
    send_message(&mut socket, "welcome", "Connected to Tondi Scan WebSocket").await?;
    
    // Handle incoming messages
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_text_message(&mut socket, &text).await {
                    eprintln!("Failed to handle message: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            _ => continue,
        }
    }
    
    Ok(())
}

async fn handle_text_message(socket: &mut WebSocket, text: &str) -> Result<()> {
    let json_msg: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| crate::error::Error::InternalServerError(format!("Invalid JSON: {}", e)))?;
    
    if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "ping" => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                send_message(socket, "pong", &format!("{}", timestamp)).await?;
            }
            "subscribe" => {
                send_message(socket, "subscribed", "Event subscription successful").await?;
            }
            "unsubscribe" => {
                send_message(socket, "unsubscribed", "Event unsubscription successful").await?;
            }
            "get_status" => {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let response = json!({
                    "type": "status",
                    "status": "connected",
                    "timestamp": timestamp
                });
                socket.send(Message::Text(response.to_string().into())).await
                    .map_err(|e| crate::error::Error::InternalServerError(format!("Failed to send message: {}", e)))?;
            }
            "get_events" => {
                let response = json!({
                    "type": "events",
                    "events": Vec::<String>::new()
                });
                socket.send(Message::Text(response.to_string().into())).await
                    .map_err(|e| crate::error::Error::InternalServerError(format!("Failed to send message: {}", e)))?;
            }
            _ => {
                send_message(socket, "error", &format!("Unknown message type: {}", msg_type)).await?;
            }
        }
    } else {
        send_message(socket, "error", "Missing message type").await?;
    }
    
    Ok(())
}

async fn send_message(socket: &mut WebSocket, msg_type: &str, message: &str) -> Result<()> {
    let response = json!({
        "type": msg_type,
        "message": message
    });
    socket.send(Message::Text(response.to_string().into())).await
        .map_err(|e| crate::error::Error::InternalServerError(format!("Failed to send message: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ctx::event_config::EventType;
    use std::str::FromStr;

    #[test]
    fn test_event_type_parsing() {
        // Test parsing valid event types
        assert!(EventType::from_str("block-added").is_ok());
        assert!(EventType::from_str("utxos-changed").is_ok());
        assert!(EventType::from_str("virtual-chain-changed").is_ok());
        
        // Test parsing invalid event types
        assert!(EventType::from_str("invalid-event").is_err());
        assert!(EventType::from_str("").is_err());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::BlockAdded.to_string(), "block-added");
        assert_eq!(EventType::UtxosChanged.to_string(), "utxos-changed");
        assert_eq!(EventType::VirtualChainChanged.to_string(), "virtual-chain-changed");
    }
}
