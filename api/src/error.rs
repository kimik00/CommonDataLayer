use rpc::error::ClientError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    ClientError(ClientError),
    #[error("Unable to parse UUID: {0}")]
    InvalidUuid(uuid::Error),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<ClientError> for ApiError {
    fn from(error: ClientError) -> Self {
        ApiError::ClientError(error)
    }
}

impl From<uuid::Error> for ApiError {
    fn from(error: uuid::Error) -> Self {
        ApiError::InvalidUuid(error)
    }
}
