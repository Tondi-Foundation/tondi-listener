use std::{io::Error as StdIoError, net::AddrParseError as StdNetAddrParseError};

use axum::response::{IntoResponse, Response as AxumResponse};
use http::StatusCode;
use tondi_listener_db::{
    diesel::{
        r2d2::PoolError as DieselR2d2PoolError,
        result::{ConnectionError as DieselConnectionError, Error as DieselError},
    },
    error::Error as TondiListenerDbError,
};
use tondi_listener_http2_client::tonic::transport::Error as TonicTransportError;

use crate::{
    ctx::config::ConfigError,
    shared::pool::Error as ClientPoolError,
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
    TondiListenerDbError(#[from] TondiListenerDbError),

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
            Self::TondiListenerDbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            Self::StdIoError(_e) => "System internal error".to_string(),
            Self::StdNetAddrParseError(e) => format!("Invalid network address: {}", e),
            Self::TonicTransportError(e) => format!("Network connection error: {}", e),
            Self::DieselR2d2PoolError(e) => format!("Database connection pool error: {}", e),
            Self::DieselConnectionError(e) => format!("Database connection error: {}", e),
            Self::DieselError(e) => format!("Database operation error: {}", e),
            Self::TondiListenerDbError(e) => format!("Database error: {}", e),
            Self::ClientPoolError(e) => format!("Client pool error: {}", e),
            Self::NotFound(msg) => format!("Resource not found: {}", msg),
            Self::Forbidden(msg) => format!("Access denied: {}", msg),
            Self::BadRequest(msg) => format!("Invalid request: {}", msg),
            Self::InternalServerError(msg) => format!("Internal server error: {}", msg),
            Self::ServiceUnavailable(msg) => format!("Service temporarily unavailable: {}", msg),
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
            Self::TondiListenerDbError(_) => "DB_OPERATION_ERROR",
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

pub type Result<T, E = Error> = std::result::Result<T, E>;
