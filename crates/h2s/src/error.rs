use tondi_scan_h2c::tonic::Status as TonicStatus;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TonicStatus(#[from] TonicStatus),

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
