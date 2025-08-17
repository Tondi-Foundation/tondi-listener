pub mod cors;
pub mod error;
pub mod limit;
pub mod trace;

use std::{convert::Infallible, time::Duration};

use axum::{
    error_handling::HandleErrorLayer, extract::Request, response::IntoResponse, routing::Route,
};
use tower::{Layer, Service, ServiceBuilder};

use crate::middleware::{cors::cors, error::handler as ErrorHandler, limit::timeout, trace::trace};

// Restrictive Service Constraints
pub trait ServiceExt: Clone + Send + Sync
where
    Self: Service<Request, Response = Self::Ret, Error = Self::Err, Future = Self::Fut>,
{
    type Ret: IntoResponse + 'static;
    type Err: Into<Infallible> + 'static;
    type Fut: Send;
}

impl<T> ServiceExt for T
where
    T: Clone + Send + Sync,
    T: Service<Request>,
    T::Response: IntoResponse + 'static,
    T::Error: Into<Infallible> + 'static,
    T::Future: Send,
{
    type Err = <Self as Service<Request>>::Error;
    type Fut = <Self as Service<Request>>::Future;
    type Ret = <Self as Service<Request>>::Response;
}

// Middleware
pub trait Middleware: Clone + Send + Sync
where
    Self: Layer<Route, Service = Self::ServiceExt>,
{
    type ServiceExt: ServiceExt;
}

impl<T> Middleware for T
where
    T: Clone + Send + Sync,
    T: Layer<Route>,
    T::Service: ServiceExt,
{
    type ServiceExt = T::Service;
}

pub fn middleware() -> impl Middleware {
    // TODO: timeout, retry, rate_limit, Compress, HandlerError
    ServiceBuilder::new()
        .layer(trace())
        .layer(cors())
        .layer(HandleErrorLayer::new(ErrorHandler))
        .load_shed()
        .layer(timeout(Duration::from_secs(15)))
}
