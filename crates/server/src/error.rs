use std::{io::Error as StdIoError, net::AddrParseError as StdNetAddrParseError};

use axum::response::{IntoResponse, Response as AxumResponse};
use nill::Nil;
use xscan_db::{
    diesel::{
        r2d2::PoolError as DieselR2d2PoolError,
        result::{ConnectionError as DieselConnectionError, Error as DieselError},
    },
    error::Error as XscanDbError,
};
use xscan_h2c::tonic::transport::Error as TonicTransportError;

use crate::shared::{data::Inner as DataInner, pool::Error as ClientPoolError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdIoError(#[from] StdIoError),

    #[error(transparent)]
    StdNetAddrParseError(#[from] StdNetAddrParseError),

    #[error(transparent)]
    TonicTransportError(#[from] TonicTransportError),

    #[error(transparent)]
    DieselR2d2PoolError(#[from] DieselR2d2PoolError),

    #[error(transparent)]
    DieselConnectionError(#[from] DieselConnectionError),

    #[error(transparent)]
    DieselError(#[from] DieselError),

    #[error(transparent)]
    XscanDbError(#[from] XscanDbError),

    #[error(transparent)]
    ClientPoolError(#[from] ClientPoolError),

    #[error("{0}")]
    Generic(String),
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self::Generic(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> AxumResponse {
        DataInner::<Nil>::fail(format!("{self}")).into()
    }
}

macro_rules! err {
    ($($arg:tt)*) => {
        Err($crate::error::Error::Generic(format!($($arg)*)))
    }
}

#[allow(unused_imports)]
pub(crate) use err;

pub type Result<T, E = Error> = std::result::Result<T, E>;
