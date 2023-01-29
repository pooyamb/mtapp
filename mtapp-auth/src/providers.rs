use axum::extract::FromRequestParts;
use uuid::Uuid;

use crate::AuthError;

#[axum::async_trait]
pub trait UserProvider {
    type Data<S: Send + Sync + 'static>: FromRequestParts<S> + Send + Sync + 'static;

    /// Given an identifier and password, init a new provider
    async fn login<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        username: &str,
        password: &str,
    ) -> Result<Uuid, AuthError>;
}

#[axum::async_trait]
pub trait GrantProvider {
    type Data<S: Send + Sync + 'static>: FromRequestParts<S> + Send + Sync + 'static;

    /// Return all the scopes for this user
    async fn scopes<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        user_id: Uuid,
    ) -> Result<Vec<String>, AuthError>;
}

#[axum::async_trait]
pub trait SessionProvider {
    type Data<S: Send + Sync + 'static>: FromRequestParts<S> + Send + Sync + 'static;

    /// Given the refresh_token, find the session and return (jti, user_id)
    async fn find<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        refresh_token: &str,
    ) -> Result<(Uuid, Uuid), AuthError>
    where
        Self: Sized;

    /// Given a user id, make a new session and return (jti, refresh_token)
    async fn make<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        user_id: Uuid,
    ) -> Result<(Uuid, String), AuthError>
    where
        Self: Sized;

    /// return a new unique identifier that will be used as jti for this session, deleting the previous one
    async fn reset_jti<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        refresh_token: &str,
    ) -> Result<Uuid, AuthError>;

    /// Given a jti, invalidate the session in a sense that it can't be used to get the user based on it
    async fn delete_by_jti<S: Send + Sync + 'static>(
        data: &Self::Data<S>,
        jti: Uuid,
    ) -> Result<(), AuthError>
    where
        Self: Sized;
}
