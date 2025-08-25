use tondi_scan_http2_client::tonic::Status as TonicStatus;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    TonicStatus(#[from] TonicStatus),

    #[error("{0}")]
    Generic(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
