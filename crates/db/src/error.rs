use diesel::result::{ConnectionError as DieselConnectionError, Error as DieselError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DieselConnectionError(#[from] DieselConnectionError),

    #[error(transparent)]
    DieselError(#[from] DieselError),

    #[error("{0}")]
    Generic(String),
    
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
