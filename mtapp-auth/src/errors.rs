use std::{error::Error, fmt};

use axum::http::StatusCode;
use basteh::BastehError;
use json_resp::JsonError;

#[derive(Debug, JsonError)]
#[json_error(internal_code = "500000 internal-error")]
pub enum AuthError {
    #[json_error(request, status = 401, code = "401000 not-authenticated")]
    Authentication,

    #[json_error(request, status = 401, code = "401001 bad-credentials")]
    Credentials,

    #[json_error(request, status = 401, code = "401002 bad-token")]
    BadToken,

    #[json_error(request, status = 403, code = "403000 not-authorized")]
    Permission,

    #[json_error(internal)]
    Configuration,

    #[json_error(internal)]
    BastehError(BastehError),

    #[json_error(internal)]
    DatabaseError(sqlx::Error),

    #[json_error(internal)]
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

impl From<BastehError> for AuthError {
    fn from(err: BastehError) -> Self {
        AuthError::BastehError(err)
    }
}
