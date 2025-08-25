use tondi_scan_db::{
    diesel::{prelude::*, r2d2::ConnectionManager, PgConnection},
    models::chain::Header,
    schema::table::THeader,
};
use nill::{Nil, nil};

use crate::{
    ctx::Context,
    error::Result,
    shared::data::Inner as DataInner,
};

pub async fn get(db: PgDb<'_>) -> Data<Header> {
    let mut conn = db.get()?;
    let select = THeader::table.select(Header::as_select());
    let header = select.order(daa_score.desc()).first(&mut conn)?;
    Ok(header.into())
}
