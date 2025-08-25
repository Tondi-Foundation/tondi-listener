use nill::{Nil, nil};
use tondi_scan_h2c::{
    protowire::{Ping, Pong},
    tonic::Status,
};
use wasm_bindgen::prelude::*;

use crate::error::Result;

pub async fn pingpong(ping: Ping) -> Result<Pong> {
    let pong = Pong {
        id: format!("Pong: {}", ping.id),
    };
    Ok(pong)
}
