use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_storage::Storage;
use axum::extract::FromRequestParts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header as TokenHeader, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthConfig, AuthError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsInner {
    pub jti: Uuid,
    pub iat: u64,
    pub exp: u64,
    #[serde(rename = "sub")]
    pub user_id: Uuid,
    pub scopes: Vec<String>,
}

/// Get the jwt Claims from extensions, it won't work outside of jwt middleware wrapped handlers.
///
/// Be advised the actual user might not exist
#[derive(Debug, Clone)]
pub struct Claims(Arc<ClaimsInner>);

impl Claims {
    pub fn new(user_id: Uuid, jti: Uuid, scopes: Vec<String>, exp: Duration) -> Self {
        Self(Arc::new(ClaimsInner {
            jti,
            user_id,
            scopes,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("UNIX_EPOCH is past")
                .as_secs(),
            exp: (SystemTime::now() + exp)
                .duration_since(UNIX_EPOCH)
                .expect("UNIX_EPOCH is past")
                .as_secs(),
        }))
    }

    pub fn generate_token(&self, secret: &str) -> String {
        encode(
            &TokenHeader::default(),
            self.0.as_ref(),
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Parameters are valid.")
    }

    pub fn from_token(token: &str, secret: &str) -> Result<Self, AuthError> {
        let inner = match decode(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(value) => Ok(value.claims),
            Err(_) => Err(AuthError::BadToken),
        }?;
        Ok(Self(Arc::new(inner)))
    }

    pub fn inner(&self) -> Arc<ClaimsInner> {
        self.0.clone()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.0.scopes.iter().any(|r| r == scope)
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = parts.extensions.get::<Self>();
        match claims {
            Some(val) => {
                let claims = val.clone();
                Ok(claims)
            }
            None => Err(AuthError::Authentication),
        }
    }
}

/// Get the jwt Claims from extensions, it won't work outside of jwt middleware wrapped handlers
/// It also adds a method to invalid current token
///
/// Be advised the actual user might not exist
pub struct ClaimsModify {
    claims: Claims,
    config: AuthConfig,
    storage: Storage,
}

impl ClaimsModify {
    pub fn get_claims(&self) -> &Claims {
        &self.claims
    }

    pub async fn invalidate(&self) -> Result<(), AuthError> {
        Ok(self
            .storage
            .scope(self.config.blacklist_scope())
            .set_expiring(self.claims.inner().jti, b"", self.config.get_token_expiry())
            .await?)
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for ClaimsModify {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;
        let config = parts
            .extensions
            .get::<AuthConfig>()
            .ok_or(AuthError::InternalError)?
            .clone();
        let storage = parts
            .extensions
            .get::<Storage>()
            .ok_or(AuthError::InternalError)?
            .clone();

        Ok(Self {
            claims,
            config,
            storage,
        })
    }
}
