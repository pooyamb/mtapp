use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::FromRequestParts;
use basteh::Storage;
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

    pub fn into_inner(self) -> Arc<ClaimsInner> {
        self.0
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

impl Deref for Claims {
    type Target = ClaimsInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Used to blacklist tokens
pub struct TokenBlacklist {
    config: AuthConfig,
    storage: Storage,
}

impl TokenBlacklist {
    pub async fn blacklist(&self, jti: Uuid) -> Result<(), AuthError> {
        Ok(self
            .storage
            .scope(self.config.blacklist_scope())
            .set_expiring(jti, b"", self.config.get_token_expiry())
            .await?)
    }
}

#[axum::async_trait]
impl<S: Sync> FromRequestParts<S> for TokenBlacklist {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<AuthConfig>()
            .ok_or(AuthError::Configuration)?
            .clone();
        let storage = parts
            .extensions
            .get::<Storage>()
            .ok_or(AuthError::Configuration)?
            .clone();

        Ok(Self { config, storage })
    }
}
