use diesel::{Queryable, Selectable, pg::Pg};
use serde::{Deserialize, Serialize};

use crate::schema::{table::THeader, tyext::hex::Hex};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = THeader, check_for_backend(Pg))]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub hash: Hex,
    pub accepted_id_merkle_root: Hex,
    pub merge_set_blues_hashes: Vec<Hex>,
    pub merge_set_reds_hashes: Option<Vec<Hex>>,
    pub selected_parent_hash: Hex,
    pub bits: i64,
    pub blue_score: i64,
    pub blue_work: Vec<u8>,
    pub daa_score: i64,
    pub hash_merkle_root: Hex,
    pub nonce: Vec<u8>,
    pub pruning_point: Hex,
    pub timestamp: i64,
    pub utxo_commitment: Hex,
    pub version: i16,
}
