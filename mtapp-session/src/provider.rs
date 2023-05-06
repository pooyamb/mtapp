use axum::{headers::UserAgent, Extension, TypedHeader};
use axum_client_ip::InsecureClientIp;
use mtapp_auth::{AuthError, SessionProvider};
use sqlx::{types::Uuid, PgPool};

use crate::models::Session;

fn extract_error(err: sqlx::Error) -> AuthError {
    match err {
        sqlx::Error::RowNotFound => AuthError::BadToken.into(),
        _ => AuthError::DatabaseError(err),
    }
}
pub struct Provider;

#[axum::async_trait]
impl SessionProvider for Provider {
    type Data<S: Send + Sync + 'static> = (
        Extension<PgPool>,
        TypedHeader<UserAgent>,
        Option<InsecureClientIp>,
    );

    async fn make<S: Send + Sync + 'static>(
        (Extension(pool), TypedHeader(user_agent), ip): &Self::Data<S>,
        user_id: Uuid,
    ) -> Result<(Uuid, String), AuthError> {
        let session = Session::create(
            user_id,
            ip.as_ref().map(|v| v.0.to_string()).unwrap_or_default(),
            user_agent.to_string(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            pool,
        )
        .await
        .map_err(extract_error)?;
        Ok((session.jti, session.refresh_token.to_string()))
    }

    async fn find<S: Send + Sync + 'static>(
        (Extension(pool), _, _): &Self::Data<S>,
        refresh_token: &str,
    ) -> Result<(Uuid, Uuid), AuthError>
    where
        Self: Sized,
    {
        let session = Session::get_by_refresh_token(
            refresh_token.parse().map_err(|_| AuthError::BadToken)?,
            pool,
        )
        .await
        .map_err(extract_error)?;

        Ok((session.jti, session.user_id))
    }

    async fn reset_jti<S: Send + Sync + 'static>(
        (Extension(pool), _, _): &Self::Data<S>,
        refresh_token: &str,
    ) -> Result<Uuid, AuthError> {
        let new_jti = Uuid::new_v4();
        Session::set_jti(
            refresh_token.parse().map_err(|_| AuthError::BadToken)?,
            new_jti,
            pool,
        )
        .await
        .map_err(extract_error)?;
        Ok(new_jti)
    }

    async fn delete_by_jti<S: Send + Sync + 'static>(
        (Extension(pool), _, _): &Self::Data<S>,
        jti: Uuid,
    ) -> Result<(), AuthError>
    where
        Self: Sized,
    {
        Session::delete_by_jti(jti, pool)
            .await
            .map_err(extract_error)?;
        Ok(())
    }
}
