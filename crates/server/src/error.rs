use std::{io::Error as StdIoError, net::AddrParseError as StdNetAddrParseError};

use axum::response::{IntoResponse, Response as AxumResponse};
use http::StatusCode;
use nill::Nil;
use tondi_scan_db::{
    diesel::{
        r2d2::PoolError as DieselR2d2PoolError,
        result::{ConnectionError as DieselConnectionError, Error as DieselError},
    },
    error::Error as TondiScanDbError,
};
use tondi_scan_http2_client::tonic::transport::Error as TonicTransportError;

use crate::{
    ctx::config::ConfigError,
    shared::{data::Inner as DataInner, pool::Error as ClientPoolError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Config error
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    // Standard library error
    #[error("IO error: {0}")]
    StdIoError(#[from] StdIoError),

    #[error("Network address parse error: {0}")]
    StdNetAddrParseError(#[from] StdNetAddrParseError),

    // Network transport error
    #[error("gRPC transport error: {0}")]
    TonicTransportError(#[from] TonicTransportError),

    // Database error
    #[error("Database connection pool error: {0}")]
    DieselR2d2PoolError(#[from] DieselR2d2PoolError),

    #[error("Database connection error: {0}")]
    DieselConnectionError(#[from] DieselConnectionError),

    #[error("Database query error: {0}")]
    DieselError(#[from] DieselError),

    #[error("Database operation error: {0}")]
    TondiScanDbError(#[from] TondiScanDbError),

    // Client pool error
    #[error("Client pool error: {0}")]
    ClientPoolError(#[from] ClientPoolError),

    // Business logic error
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    Forbidden(String),

    #[error("Invalid request parameters: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Service temporarily unavailable: {0}")]
    ServiceUnavailable(String),

    // Generic error
    #[error("{0}")]
    Generic(String),
}

impl Error {
    /// Get HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::StdIoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::StdNetAddrParseError(_) => StatusCode::BAD_REQUEST,
            Self::TonicTransportError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::DieselR2d2PoolError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::DieselConnectionError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::DieselError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TondiScanDbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ClientPoolError(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Generic(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(e) => format!("System configuration error: {}", e),
            Self::StdIoError(e) => "System internal error".to_string(),
            Self::StdNetAddrParseError(e) => format!("Invalid address format: {}", e),
            Self::TonicTransportError(_) => "Service temporarily unavailable, please try again later".to_string(),
            Self::DieselR2d2PoolError(_) => "Database service temporarily unavailable, please try again later".to_string(),
            Self::DieselConnectionError(_) => "Database connection failed, please try again later".to_string(),
            Self::DieselError(_) => "Database operation failed".to_string(),
            Self::TondiScanDbError(e) => format!("Database error: {}", e),
            Self::ClientPoolError(_) => "Client service temporarily unavailable, please try again later".to_string(),
            Self::NotFound(resource) => format!("Requested resource '{}' does not exist", resource),
            Self::Forbidden(reason) => format!("Access denied: {}", reason),
            Self::BadRequest(details) => format!("Invalid request parameters: {}", details),
            Self::InternalServerError(_) => "Server internal error, please try again later".to_string(),
            Self::ServiceUnavailable(_) => "Service temporarily unavailable, please try again later".to_string(),
            Self::Generic(msg) => msg.clone(),
        }
    }

    /// Get error code (for logging and debugging)
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG_ERROR",
            Self::StdIoError(_) => "IO_ERROR",
            Self::StdNetAddrParseError(_) => "ADDR_PARSE_ERROR",
            Self::TonicTransportError(_) => "GRPC_TRANSPORT_ERROR",
            Self::DieselR2d2PoolError(_) => "DB_POOL_ERROR",
            Self::DieselConnectionError(_) => "DB_CONNECTION_ERROR",
            Self::DieselError(_) => "DB_QUERY_ERROR",
            Self::TondiScanDbError(_) => "DB_OPERATION_ERROR",
            Self::ClientPoolError(_) => "CLIENT_POOL_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::Generic(_) => "GENERIC_ERROR",
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Generic(err)
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Generic(err.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        let status = self.status_code();
        let error_response = serde_json::json!({
            "error": {
                "code": self.error_code(),
                "message": self.user_message(),
                "status": status.as_u16()
            }
        });

        (status, axum::Json(error_response)).into_response()
    }
}

// Convenient error construction macros
macro_rules! err {
    ($($arg:tt)*) => {
        Err($crate::error::Error::Generic(format!($($arg)*)))
    }
}

macro_rules! not_found {
    ($($arg:tt)*) => {
        Err($crate::error::Error::NotFound(format!($($arg)*)))
    }
}

macro_rules! bad_request {
    ($($arg:tt)*) => {
        Err($crate::error::Error::BadRequest(format!($($arg)*)))
    }
}

macro_rules! forbidden {
    ($($arg:tt)*) => {
        Err($crate::error::Error::Forbidden(format!($($arg)*)))
    }
}

macro_rules! internal_error {
    ($($arg:tt)*) => {
        Err($crate::error::Error::InternalServerError(format!($($arg)*)))
    }
}

#[allow(unused_imports)]
pub(crate) use {bad_request, err, forbidden, internal_error, not_found};

pub type Result<T, E = Error> = std::result::Result<T, E>;
