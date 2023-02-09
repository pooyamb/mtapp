use std::fmt;

use axum::http::StatusCode;
use json_resp::JsonError;

#[derive(Debug, JsonError)]
#[json_error(internal_code = "500000 internal-error")]
pub enum ScopeError {
    #[json_error(request, status = 404, code = "404001 resource-not-found")]
    NotFound,

    #[json_error(request, status = 409, code = "409001 already-exist")]
    DuplicateField(&'static str),

    #[json_error(internal)]
    DatabaseError(sqlx::Error),

    #[json_error(internal)]
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
