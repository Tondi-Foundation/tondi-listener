use axum::{
    extract::{Path, State},
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

/// Get transaction by ID
pub async fn get_transaction_by_id(
    Path(transaction_id): Path<String>,
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get transaction by ID
    let result: Result<Option<Tx>, diesel::result::Error> = conn
        .transaction(|conn| {
            TTx::table
                .filter(TTx::transaction_id.eq(transaction_id.clone()))
                .first::<Tx>(conn)
                .optional()
        });

    match result {
        Ok(Some(tx)) => {
            // Get transaction outputs
            let outputs_result: Result<Vec<TxOu>, diesel::result::Error> = conn
                .transaction(|conn| {
                    TTxOu::table
                        .filter(TTxOu::transaction_id.eq(transaction_id.clone()))
                        .load::<TxOu>(conn)
                });

            let outputs = match outputs_result {
                Ok(outputs) => outputs,
                Err(e) => {
                    log::warn!("Failed to fetch outputs for transaction {}: {}", transaction_id, e);
                    Vec::new()
                }
            };

            let response = serde_json::json!({
                "success": true,
                "data": {
                    "transaction": {
                        "transaction_id": tx.transaction_id,
                        "hash": tx.hash,
                        "subnetwork_id": tx.subnetwork_id,
                        "mass": tx.mass,
                        "payload": tx.payload,
                        "block_time": tx.block_time
                    },
                    "outputs": outputs.into_iter().map(|output| {
                        serde_json::json!({
                            "index": output.index,
                            "amount": output.amount,
                            "script_public_key_address": output.script_public_key_address,
                            "block_time": output.block_time
                        })
                    }).collect::<Vec<_>>()
                }
            });
            Ok(Json(response))
        }
        Ok(None) => {
            Err((
                StatusCode::NOT_FOUND,
                format!("Transaction not found: {}", transaction_id),
            ))
        }
        Err(e) => {
            log::error!("Failed to fetch transaction {}: {}", transaction_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch transaction: {}", e),
            ))
        }
    }
}

/// Get transaction outputs by transaction ID
pub async fn get_transaction_outputs(
    Path(transaction_id): Path<String>,
    State(pool): State<DieselPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let conn = pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database connection error: {}", e),
        )
    })?;

    // Get transaction outputs by transaction ID
    let result: Result<Vec<TxOu>, diesel::result::Error> = conn
        .transaction(|conn| {
            TTxOu::table
                .filter(TTxOu::transaction_id.eq(transaction_id.clone()))
                .load::<TxOu>(conn)
        });

    match result {
        Ok(outputs) => {
            let response = serde_json::json!({
                "success": true,
                "data": {
                    "transaction_id": transaction_id,
                    "outputs": outputs.into_iter().map(|output| {
                        serde_json::json!({
                            "index": output.index,
                            "amount": output.amount,
                            "script_public_key_address": output.script_public_key_address,
                            "block_time": output.block_time
                        })
                    }).collect::<Vec<_>>()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("Failed to fetch outputs for transaction {}: {}", transaction_id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch transaction outputs: {}", e),
            ))
        }
    }
}
