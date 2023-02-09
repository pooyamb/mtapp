use std::fmt;

use axum::http::StatusCode;
use json_resp::JsonError;

#[derive(Debug, JsonError)]
#[json_error(internal_code = "500000 internal-error")]
pub enum SessionError {
    #[json_error(request, status = 404, code = "404001 resource-not-found")]
    NotFound,

    #[json_error(internal)]
    DatabaseError(sqlx::Error),

    #[json_error(internal)]
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
