use diesel::{Queryable, Selectable, pg::Pg};
use serde::{Deserialize, Serialize};

use crate::schema::{
    table::{TTx, TTxOu},
    tyext::hex::Hex,
};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = TTx, check_for_backend(Pg))]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    pub transaction_id: Hex,
    pub subnetwork_id: i32,
    pub hash: Hex,
    pub mass: Option<i32>,
    pub payload: Option<Vec<u8>>,
    pub block_time: i64,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = TTxOu, check_for_backend(Pg))]
#[serde(rename_all = "camelCase")]
pub struct TxOu {
    pub transaction_id: Hex,
    pub index: i16,
    pub amount: i64,
    pub script_public_key: Vec<u8>,
    pub script_public_key_address: String,
    pub block_time: i64,
}
