use xscan_db::{
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper},
    models::transaction::Tx,
    schema::table::TTx,
};

use crate::{ctx::pg_database::PgDb, shared::data::Data};

pub async fn get(db: PgDb<'_>) -> Data<Tx> {
    let mut conn = db.get()?;
    let select = TTx::table.select(Tx::as_select());
    let tx = select.order(TTx::block_time.desc()).first(&mut conn)?;
    Ok(tx.into())
}
