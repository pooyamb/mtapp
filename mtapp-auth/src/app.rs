use std::marker::PhantomData;
use std::time::Duration;

use axum::middleware::from_fn;
use axum::routing::post;
use axum::Router;
use mtapp::{App, Configuration};
use secrecy::{ExposeSecret, Secret};

use crate::handlers::*;
use crate::middleware::jwt_claims;
use crate::openapi::get_open_api;
use crate::providers::{GrantProvider, SessionProvider, UserProvider};

const TOKENEXPIRY: u64 = 24 * 60 * 60;

#[derive(Clone)]
pub struct AuthConfig {
    // The scope(tree name) used to store the blacklisted tokens in storage
    blacklist_scope: String,

    // Time to live for JWT tokens
    token_expiry: Duration,

    // Secret used to sign jwt tokens
    secret: Secret<String>,
}

impl AuthConfig {
    pub fn new(storage_scope: &str, token_expiry: u64, secret: String) -> Self {
        AuthConfig {
            blacklist_scope: String::from(storage_scope),
            token_expiry: Duration::from_secs(token_expiry),
            secret: Secret::new(secret),
        }
    }

    pub fn blacklist_scope(&self) -> &str {
        &self.blacklist_scope
    }

    pub fn expose_secret(&self) -> &str {
        &self.secret.expose_secret()
    }

    pub fn get_token_expiry(&self) -> Duration {
        self.token_expiry
    }
}

#[derive(Clone)]
pub struct AuthApp<U, S, G> {
    config: AuthConfig,

    _phantom: PhantomData<dyn Fn() -> (U, S, G) + Sync + Send>,
}

impl<U, S, G> AuthApp<U, S, G> {
    pub fn new(secret: String) -> Self {
        Self {
            config: AuthConfig::new("storage_scope", TOKENEXPIRY, secret),
            _phantom: PhantomData,
        }
    }

    pub fn with_config(config: AuthConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

impl<U, S, G> App for AuthApp<U, S, G>
where
    U: UserProvider + 'static + Send + Sync,
    S: SessionProvider + 'static + Send + Sync,
    G: GrantProvider + 'static + Send + Sync,
{
    fn name(&self) -> &'static str {
        "auth"
    }

    fn configure(&mut self, cfg: &mut Configuration) {
        let config = self.config.clone();
        cfg.global_state(move |ext| {
            ext.insert(config.clone());
        })
        .base_router(|router| 
            // Register auth middleware
            router.layer(from_fn(jwt_claims))
        );
    }

    fn public_routes(&mut self, path_prefix: &str) -> Option<Router> {
        Some(
            Router::new()
                .route(&format!("{}/login", path_prefix), post(login::<U, S, G>))
                .route(&format!("{}/refresh", path_prefix), post(refresh::<S, G>))
                .route(&format!("{}/logout", path_prefix), post(logout::<U, S>)),
        )
    }

    fn internal_routes(&mut self, path_prefix: &str) -> Option<Router> {
        Some(
            Router::new()
                .route(&format!("{}/login", path_prefix), post(login::<U, S, G>))
                .route(&format!("{}/refresh", path_prefix), post(refresh::<S, G>))
                .route(&format!("{}/logout", path_prefix), post(logout::<U, S>)),
        )
    }

    fn public_openapi(&mut self, path: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(get_open_api(path))
    }

    fn internal_openapi(&mut self, path: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(get_open_api(path))
    }
}
