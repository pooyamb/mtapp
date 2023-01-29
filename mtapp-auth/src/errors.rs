use std::{error::Error, fmt};

use actix_storage::StorageError;
use axum::http::StatusCode;
use json_response::ApiError;

#[derive(Debug, ApiError)]
pub enum AuthError {
    #[request_error(
        status = StatusCode::UNAUTHORIZED,
        code = "40100 not-authenticated",
    )]
    Authentication,
    #[request_error(
        status = StatusCode::UNAUTHORIZED,
        code = "40101 bad-credentials",
    )]
    Credentials,
    #[request_error(
        status = StatusCode::UNAUTHORIZED, code = "40102 bad-token")]
    BadToken,
    #[request_error(
        status = StatusCode::FORBIDDEN,
        code = "40300 not-authorized",
    )]
    Permission,
    #[internal_error]
    InternalError,
    #[internal_error]
    Other(Box<dyn Error + Send>),
}

impl AuthError {
    pub fn other<E: Error + Send + 'static>(e: E) -> Self {
        Self::Other(Box::new(e))
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UserError")
    }
}

impl From<StorageError> for AuthError {
    fn from(err: StorageError) -> Self {
        log::error!("Auth App: Internal Error(StorageError): {}", err);

        AuthError::InternalError
    }
}
