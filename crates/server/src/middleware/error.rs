use axum::BoxError;
use nill::Nil;

use crate::shared::data::Inner as DataInner;

// TODO: Remove Layer Constraints Error: Into<Infallible>, Reduce Service Clone
pub async fn handler(err: BoxError) -> DataInner<Nil> {
    DataInner::fail(format!("{err}"))
}
