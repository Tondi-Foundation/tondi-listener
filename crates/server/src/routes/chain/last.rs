use xscan_db::{
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper},
    models::chain::Header,
    schema::table::{THeader, THeader::daa_score},
};

use crate::{ctx::pg_database::PgDb, shared::data::Data};

pub async fn get(db: PgDb<'_>) -> Data<Header> {
    let mut conn = db.get()?;
    let select = THeader::table.select(Header::as_select());
    let header = select.order(daa_score.desc()).first(&mut conn)?;
    Ok(header.into())
}
