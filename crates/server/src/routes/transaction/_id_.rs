use axum::extract::Path;
use tondi_scan_db::{
    diesel::{prelude::*, r2d2::ConnectionManager, PgConnection},
    models::transaction::Transaction,
    schema::table::TTransaction,
};
use nill::{Nil, nil};

use crate::{
    ctx::Context,
    error::Result,
    shared::data::Inner as DataInner,
};

pub async fn get(Path(id): Path<String>, db: PgDb<'_>) -> Data<Tx> {
    let mut conn = db.get()?;
    let select = TTx::table.select(Tx::as_select());
    let tx_id = Hash256::from_hex(id)?;
    let filter = TTx::transaction_id.eq(tx_id);
    let tx = select.filter(filter).first(&mut conn)?;
    Ok(tx.into())
}
