use std::fmt;

use axum::http::StatusCode;
use json_response::ApiError;

#[derive(Debug, ApiError)]
pub enum SessionError {
    #[request_error(status=StatusCode::NOT_FOUND, code="404001 resource-not-found")]
    NotFound,
    #[internal_error]
    DatabaseError(sqlx::Error),
    #[internal_error]
    InternalError,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionError")
    }
}

impl From<sqlx::Error> for SessionError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => SessionError::NotFound,
            _ => SessionError::DatabaseError(err),
        }
    }
}
