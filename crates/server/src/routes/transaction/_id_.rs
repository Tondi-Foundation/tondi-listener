use axum::extract::Path;
use xscan_db::{
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper},
    models::transaction::Tx,
    schema::{
        table::TTx,
        tyext::hash::{FromHex, Hash256},
    },
};

use crate::{ctx::pg_database::PgDb, shared::data::Data};

pub async fn get(Path(id): Path<String>, db: PgDb<'_>) -> Data<Tx> {
    let mut conn = db.get()?;
    let select = TTx::table.select(Tx::as_select());
    let tx_id = Hash256::from_hex(id)?;
    let filter = TTx::transaction_id.eq(tx_id);
    let tx = select.filter(filter).first(&mut conn)?;
    Ok(tx.into())
}
