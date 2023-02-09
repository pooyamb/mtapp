use std::fmt;

use axum::http::StatusCode;
use json_resp::JsonError;

#[derive(Debug, JsonError)]
pub enum GrantError {
    #[json_error(request, status = 404, code = "404001 resource-not-found")]
    NotFound,

    #[json_error(request, status = 409, code = "409003 grant-already-exist")]
    AlreadyExist,

    #[json_error(internal)]
    DatabaseError(sqlx::Error),

    #[json_error(internal)]
    UnknownConstaintError(Box<sqlx::postgres::PgDatabaseError>),
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
                    Some("grants_uniq") => GrantError::AlreadyExist,
                    _ => GrantError::UnknownConstaintError(pg_error),
                }
            }
            _ => GrantError::DatabaseError(err),
        }
    }
}
