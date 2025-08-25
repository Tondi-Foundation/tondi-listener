use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use tondi_listener_db::{
    models::transaction::{Tx, TxOu},
    schema::table::{TTx, TTxOu},
    DieselPool,
};
use diesel::prelude::*;
use serde_json::Value;

use crate::error::Result;

/// Get the latest transaction information
pub async fn get_last_transaction(
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get the latest transaction by block time
    let result: Result<Tx, diesel::result::Error> = conn
        .transaction(|conn| {
            TTx::table
                .order(TTx::block_time.desc())
                .first::<Tx>(conn)
        });

    match result {
        Ok(tx) => {
            let response = serde_json::json!({
                "success": true,
                "data": {
                    "transaction_id": tx.transaction_id,
                    "hash": tx.hash,
                    "subnetwork_id": tx.subnetwork_id,
                    "mass": tx.mass,
                    "block_time": tx.block_time
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch latest transaction: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch latest transaction: {}", e),
            ))
        }
    }
}

/// Get transaction statistics
pub async fn get_transaction_stats(
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get transaction statistics
    let result: Result<(i64, i64, i64), diesel::result::Error> = conn
        .transaction(|conn| {
            let total_transactions = TTx::table.count().get_result::<i64>(conn)?;
            let total_outputs = TTxOu::table.count().get_result::<i64>(conn)?;
            let latest_block_time = TTx::table
                .select(TTx::block_time)
                .order(TTx::block_time.desc())
                .first::<i64>(conn)
                .optional()?
                .unwrap_or(0);
            
            Ok((total_transactions, total_outputs, latest_block_time))
        });

    match result {
        Ok((total_transactions, total_outputs, latest_block_time)) => {
            let response = serde_json::json!({
                "success": true,
                "data": {
                    "total_transactions": total_transactions,
                    "total_outputs": total_outputs,
                    "latest_block_time": latest_block_time
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch transaction stats: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch transaction stats: {}", e),
            ))
        }
    }
}
