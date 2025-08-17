use diesel::result::{ConnectionError as DieselConnectionError, Error as DieselError};
use hex::Error as HexError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    DieselConnectionError(#[from] DieselConnectionError),

    #[error(transparent)]
    DieselError(#[from] DieselError),

    #[error(transparent)]
    HexError(#[from] HexError),

    #[error("{0}")]
    Generic(String),
}

macro_rules! err {
    ($($arg:tt)*) => {
        Err($crate::error::Error::Generic(format!($($arg)*)))
    }
}

#[allow(unused_imports)]
pub(crate) use err;

pub type Result<T, E = Error> = std::result::Result<T, E>;
