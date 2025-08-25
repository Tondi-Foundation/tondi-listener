use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pong {
    pub id: String,
}

pub async fn pingpong(ping: Ping) -> Result<Pong> {
    let pong = Pong {
        id: format!("Pong: {}", ping.id),
    };
    Ok(pong)
}
