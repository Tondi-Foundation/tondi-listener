pub mod cors;
pub mod error;
pub mod limit;
pub mod trace;
pub mod security;

use std::{convert::Infallible, time::Duration};

use axum::{
    error_handling::HandleErrorLayer, extract::Request, response::IntoResponse, routing::Route,
};
use tower::{Layer, Service, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::{
    ctx::config::{Config, SecurityConfig},
    error::Error,
    middleware::{cors::cors, error::handler as ErrorHandler, limit::timeout, trace::trace, security::rate_limit},
};

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

pub fn middleware(config: &Config) -> impl Middleware {
    let security = &config.security;
    
    ServiceBuilder::new()
        // Basic middleware
        .layer(TraceLayer::new_for_http())
        .layer(trace())
        
        // Security middleware
        .layer(cors(&config.cors))
        .layer(rate_limit(security.rate_limit))
        .layer(RequestBodyLimitLayer::new(security.max_body_size))
        
        // Performance middleware
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(security.timeout)))
        
        // Error handling
        .layer(HandleErrorLayer::new(ErrorHandler))
        
        // Load balancing
        .load_shed()
        
        // Timeout handling
        .layer(timeout(Duration::from_secs(security.timeout)))
}

/// Development environment middleware configuration
pub fn development_middleware() -> impl Middleware {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(trace())
        .layer(cors(&crate::ctx::config::CorsConfig::default()))
        .layer(HandleErrorLayer::new(ErrorHandler))
        .load_shed()
        .layer(timeout(Duration::from_secs(30)))
}

/// Production environment middleware configuration
pub fn production_middleware(config: &Config) -> impl Middleware {
    let security = &config.security;
    
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(trace())
        .layer(crate::middleware::cors::strict_cors())
        .layer(rate_limit(security.rate_limit))
        .layer(RequestBodyLimitLayer::new(security.max_body_size))
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(security.timeout)))
        .layer(HandleErrorLayer::new(ErrorHandler))
        .load_shed()
        .layer(timeout(Duration::from_secs(security.timeout)))
}
