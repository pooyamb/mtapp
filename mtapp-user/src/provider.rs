use axum::Extension;
use mtapp_auth::{AuthError, UserProvider};
use sqlx::{types::Uuid, PgPool};

use crate::models::User;

fn extract_error(err: sqlx::Error) -> AuthError {
    match err {
        sqlx::Error::RowNotFound => AuthError::Credentials,
        _ => AuthError::other(err),
    }
}

pub struct Provider;

#[axum::async_trait]
impl UserProvider for Provider {
    type Data<S: Sync + Send + 'static> = Extension<PgPool>;

    async fn login<S: Send + Sync + 'static>(
        Extension(pool): &Extension<PgPool>,
        username: &str,
        password: &str,
    ) -> Result<Uuid, AuthError> {
        let user = User::get_by_username(username, pool)
            .await
            .map_err(extract_error)?;

        if user.check_password(password) {
            User::update_login_timestamp(user.id, pool)
                .await
                .map_err(extract_error)?;
            Ok(user.id)
        } else {
            Err(AuthError::Credentials)
        }
    }
}
