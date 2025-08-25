use tondi_scan_db::{
    diesel::{prelude::*, r2d2::ConnectionManager, PgConnection},
    models::transaction::Transaction,
    schema::table::TTransaction,
};
use axum::extract::Query;
use serde::Deserialize;
use nill::{Nil, nil};

use crate::{
    ctx::Context,
    error::Result,
    shared::data::Inner as DataInner,
};

pub async fn get(db: PgDb<'_>) -> Data<Tx> {
    let mut conn = db.get()?;
    let select = TTx::table.select(Tx::as_select());
    let tx = select.order(TTx::block_time.desc()).first(&mut conn)?;
    Ok(tx.into())
}
