use std::marker::PhantomData;
use std::time::Duration;

use axum::http::Extensions;
use axum::middleware::from_fn;
use axum::routing::post;
use axum::Router;
use mtapp::App;
use secrecy::{ExposeSecret, Secret};

use crate::handlers::*;
use crate::middleware::jwt_claims;
use crate::providers::{GrantProvider, SessionProvider, UserProvider};

const TOKENEXPIRY: u64 = 60 * 60 * 5;

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

    fn public_routes(&mut self) -> Option<Router> {
        Some(
            Router::new()
                .route("/login", post(login::<U, S, G>))
                .route("/refresh", post(refresh::<S, G>))
                .merge(Router::new().route("/logout", post(logout::<U, S>))),
        )
    }

    fn internal_routes(&mut self) -> Option<Router> {
        Some(
            Router::new()
                .route("/login", post(login::<U, S, G>))
                .route("/refresh", post(refresh::<S, G>))
                .merge(Router::new().route("/logout", post(logout::<U, S>))),
        )
    }

    fn data_register(&self, ext: &mut Extensions) {
        ext.insert(self.config.clone());
    }

    fn _mod_base_router(&self, router: Router) -> Router {
        router.layer(from_fn(jwt_claims))
    }
}
