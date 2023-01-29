use std::{error::Error, fmt};

use axum::http::StatusCode;
use json_response::ApiError;

#[derive(Debug, ApiError)]
pub enum GrantError {
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

impl fmt::Display for GrantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GrantError")
    }
}

impl From<sqlx::Error> for GrantError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => GrantError::NotFound,
            sqlx::Error::Database(db_err) => {
                // It's hacky and should be converted into a more general way possiblity converted to ValidationError
                let pg_error = db_err.downcast::<sqlx::postgres::PgDatabaseError>();
                match pg_error.constraint() {
                    _ => {
                        log::error!(
                            "Grant App: Internal Error(UnknownConstaintError): {}",
                            pg_error
                        );

                        GrantError::UnknownConstaintError(pg_error)
                    }
                }
            }
            _ => {
                log::error!("User App: Internal Error(DatabaseError): {}", err);
                GrantError::DatabaseError(err)
            }
        }
    }
}
