use std::{error::Error, fmt};

use axum::http::StatusCode;
use basteh::StorageError;
use json_resp::JsonError;

#[derive(Debug, JsonError)]
pub enum UserError {
    #[json_error(request, status = 404, code = "404001 resource-not-found")]
    NotFound,

    #[json_error(request, status = 409, code = "409001 already-exist")]
    DuplicateField(&'static str),

    #[json_error(request, status = 409, code = "409002 validation-error")]
    ValidationError(validator::ValidationErrors),

    #[json_error(internal)]
    DatabaseError(sqlx::Error),

    #[json_error(internal)]
    UnknownConstaintError(Box<sqlx::postgres::PgDatabaseError>),

    #[json_error(internal)]
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
