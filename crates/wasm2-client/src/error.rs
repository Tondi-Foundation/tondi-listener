use thiserror::Error;
use tondi_scan_http2_client::tonic::Status as TonicStatus;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TonicStatus(#[from] TonicStatus),

    #[error("{0}")]
    Generic(String),
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

pub type Result<T, E = Error> = std::result::Result<T, E>;
