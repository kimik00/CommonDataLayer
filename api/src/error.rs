use rpc::error::ClientError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    ClientError(#[from] ClientError),
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

pub type Result<T, E = ApiError> = std::result::Result<T, E>;
