use crate::{
    extensions::client_pool::ClientPool,
    shared::data::Data,
};
use tondi_scan_db::models::transaction::Tx;

pub async fn get(_db: axum::extract::State<ClientPool>) -> Data<Tx> {
    // TODO: 实现数据库访问逻辑
    Err(crate::error::Error::InternalServerError(
        "Transaction endpoint not yet implemented".to_string()
    ))
}
