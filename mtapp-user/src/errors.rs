use std::{error::Error, fmt};

use axum::http::StatusCode;
use basteh::StorageError;
use json_response::ApiError;

#[derive(Debug, ApiError)]
pub enum UserError {
    #[request_error(status=StatusCode::CONFLICT, code="409000 conflict-error")]
    ValidationError(validator::ValidationErrors),
    #[request_error(status=StatusCode::NOT_FOUND, code="404001 resource-not-found")]
    NotFound,
    #[request_error(status=StatusCode::CONFLICT, code="409001 already-exist")]
    DuplicateField(&'static str),
    #[internal_error]
    DatabaseError(sqlx::Error),
    #[internal_error]
    UnknownConstaintError(Box<sqlx::postgres::PgDatabaseError>),
    #[internal_error]
    Other(Box<dyn Error + Send>),
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("UserError")
    }
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UserError::NotFound,
            sqlx::Error::Database(db_err) => {
                // It's hacky and should be converted into a more general way possiblity converted to ValidationError
                let pg_error = db_err.downcast::<sqlx::postgres::PgDatabaseError>();
                match pg_error.constraint() {
                    Some("username_uniq") => UserError::DuplicateField("username"),
                    Some("email_uniq") => UserError::DuplicateField("email"),
                    _ => UserError::UnknownConstaintError(pg_error),
                }
            }
            _ => UserError::DatabaseError(err),
        }
    }
}

impl From<validator::ValidationErrors> for UserError {
    fn from(err: validator::ValidationErrors) -> Self {
        UserError::ValidationError(err)
    }
}

impl From<StorageError> for UserError {
    fn from(err: StorageError) -> Self {
        UserError::Other(Box::new(err))
    }
}
