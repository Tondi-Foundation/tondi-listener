use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use tondi_listener_db::{
    models::chain::Header,
    schema::table::THeader,
    DieselPool,
};
use diesel::prelude::*;
use serde_json::Value;

use crate::error::Result;

/// Get the latest block header information
pub async fn get_last_header(
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get the latest header by timestamp
    let result: Result<Header, diesel::result::Error> = conn
        .transaction(|conn| {
            THeader::table
                .order(THeader::timestamp.desc())
                .first::<Header>(conn)
        });

    match result {
        Ok(header) => {
            let response = serde_json::json!({
                "success": true,
                "data": {
                    "hash": header.hash,
                    "timestamp": header.timestamp,
                    "blue_score": header.blue_score,
                    "daa_score": header.daa_score,
                    "bits": header.bits,
                    "version": header.version
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch latest header: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch latest header: {}", e),
            ))
        }
    }
}

/// Get chain statistics
pub async fn get_chain_stats(
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get chain statistics
    let result: Result<(i64, i64, i64), diesel::result::Error> = conn
        .transaction(|conn| {
            let total_blocks = THeader::table.count().get_result::<i64>(conn)?;
            let latest_timestamp = THeader::table
                .select(THeader::timestamp)
                .order(THeader::timestamp.desc())
                .first::<i64>(conn)
                .optional()?
                .unwrap_or(0);
            let latest_blue_score = THeader::table
                .select(THeader::blue_score)
                .order(THeader::blue_score.desc())
                .first::<i64>(conn)
                .optional()?
                .unwrap_or(0);
            
            Ok((total_blocks, latest_timestamp, latest_blue_score))
        });

    match result {
        Ok((total_blocks, latest_timestamp, latest_blue_score)) => {
            let response = serde_json::json!({
                "success": true,
                "data": {
                    "total_blocks": total_blocks,
                    "latest_timestamp": latest_timestamp,
                    "latest_blue_score": latest_blue_score
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch chain stats: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch chain stats: {}", e),
            ))
        }
    }
}
