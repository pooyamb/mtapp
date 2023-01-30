use std::fmt;

use axum::http::StatusCode;
use json_response::ApiError;

#[derive(Debug, ApiError)]
pub enum ScopeError {
    #[request_error(status=StatusCode::NOT_FOUND, code="404001 resource-not-found")]
    NotFound,
    #[request_error(status=StatusCode::CONFLICT, code="409001 already-exist")]
    DuplicateField(&'static str),
    #[internal_error]
    DatabaseError(sqlx::Error),
    #[internal_error]
    UnknownConstaintError(Box<sqlx::postgres::PgDatabaseError>),
}

impl fmt::Display for ScopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ScopeError")
    }
}

impl From<sqlx::Error> for ScopeError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ScopeError::NotFound,
            sqlx::Error::Database(db_err) => {
                // It's hacky and should be converted into a more general way possiblity converted to ValidationError
                let pg_error = db_err.downcast::<sqlx::postgres::PgDatabaseError>();
                match pg_error.constraint() {
                    Some("name_uniq") => ScopeError::DuplicateField("name"),
                    _ => ScopeError::UnknownConstaintError(pg_error),
                }
            }
            _ => ScopeError::DatabaseError(err),
        }
    }
}
