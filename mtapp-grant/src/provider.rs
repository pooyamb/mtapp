use axum::Extension;
use mtapp_auth::{AuthError, GrantProvider};
use sqlx::{types::Uuid, PgPool};

use crate::models::Grant;

pub struct Provider;

#[axum::async_trait]
impl GrantProvider for Provider {
    type Data<S: Sync + Send + 'static> = Extension<PgPool>;

    async fn scopes<S: Send + Sync + 'static>(
        Extension(pool): &Extension<PgPool>,
        user_id: Uuid,
    ) -> Result<Vec<String>, AuthError> {
        Ok(Grant::get_grants(user_id, pool)
            .await
            .map_err(AuthError::other)?
            .into_iter()
            .map(|row| row.scope_name)
            .collect())
    }
}
