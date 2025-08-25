use std::error::Error as StdError;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use nill::Nil;
use tokio::task::JoinSet;
use tondi_rpc_core::Notification;
use tondi_utils::channel::Receiver;

use crate::{ctx::Context, error::Error, extensions::client_pool::ClientPool};

#[derive(Debug)]
pub struct State {
    pub rt: JoinSet<Result<Nil, Error>>,
    pub consumer: Receiver<Notification>,
}

impl State {
    pub fn new(consumer: Receiver<Notification>) -> Self {
        Self { rt: JoinSet::new(), consumer }
    }
}

#[derive(Debug)]
pub struct Rejection {
    status: StatusCode,
    reject: String,
}

impl Rejection {
    pub const fn new(reject: String) -> Self {
        Self { status: StatusCode::INTERNAL_SERVER_ERROR, reject }
    }
}

impl<E> From<E> for Rejection
where
    E: StdError,
{
    fn from(err: E) -> Self {
        Self::new(format!("{err}"))
    }
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        let Self { status, reject } = self;
        (status, reject).into_response()
    }
}

impl FromRequestParts<Context> for State {
    type Rejection = Rejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Context,
    ) -> Result<Self, Self::Rejection> {
        let client_pool = ClientPool::from_request_parts(parts, state).await?;
        let client = client_pool.get().await?;
        let consumer = client.consumer.rx.clone();
        Ok(Self::new(consumer))
    }
}
