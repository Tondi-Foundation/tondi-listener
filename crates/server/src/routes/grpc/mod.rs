pub mod grpc_call;
pub mod grpc_return;

use axum::extract::Json;

use crate::{
    error::err,
    extensions::client_pool::ClientPool,
    routes::grpc::{grpc_call::GrpcCall, grpc_return::GrpcReturn},
    shared::data::Data,
};

pub async fn post(client_pool: ClientPool, Json(grpc_call): Json<GrpcCall>) -> Data<GrpcReturn> {
    let client = client_pool.get().await?;
    let (op, params) = grpc_call.into();
    let response = client.call(op, params).await?;
    let Some(payload) = response.payload else {
        return err!("Grpc Playload Return None");
    };
    Ok(GrpcReturn::try_from(payload)?.into())
}
