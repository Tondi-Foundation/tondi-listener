pub mod grpc_call;
pub mod grpc_return;

use axum::extract::Json;

use crate::{
    error::Error as AppError,
    extensions::client_pool::ClientPool,
    routes::grpc::{grpc_call::GrpcCall, grpc_return::GrpcReturn},
    shared::data::Data,
};

pub async fn post(_client_pool: ClientPool, Json(_grpc_call): Json<GrpcCall>) -> Data<GrpcReturn> {
    // 暂时简化gRPC调用，因为具体的类型需要根据实际的API来实现
    // TODO: 实现真正的gRPC调用逻辑
    Err(AppError::InternalServerError(
        "gRPC calls not yet implemented".to_string()
    ))
}
