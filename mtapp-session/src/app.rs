// use mtapp_auth::JwtMiddleware;

use axum::{routing::get, Router};
use mtapp::include_migrations_dir;
use mtapp::App;
use mtapp_auth::ClaimCheck;
use mtapp_auth::Claims;
use utoipa::OpenApi;

use crate::admin;
use crate::handlers;
use crate::openapi::{InternalSessionOpenApi, PublicSessionOpenApi};

#[derive(Default, Clone)]
pub struct SessionApp {}

impl SessionApp {
    pub fn new() -> Self {
        SessionApp {}
    }
}

impl App for SessionApp {
    fn name(&self) -> &'static str {
        "mtapp-session"
    }

    fn public_routes(&mut self, path_prefix: &str) -> Option<Router> {
        Some(
            Router::new()
                .route(&format!("{}/", path_prefix), get(handlers::list))
                .route(&format!("{}/current", path_prefix), get(handlers::get))
                .route(
                    &format!("{}/:session_id", path_prefix),
                    get(handlers::get).delete(handlers::delete),
                )
                .layer(ClaimCheck::new(|claims: Option<Claims>| claims.is_some())),
        )
    }

    fn internal_routes(&mut self, path_prefix: &str) -> Option<Router> {
        Some(
            Router::new()
                .route(
                    &format!("{}/", path_prefix),
                    get(admin::list).delete(admin::batch_delete),
                )
                .route(
                    &format!("{}/:session_id", path_prefix),
                    get(admin::get).delete(admin::delete),
                )
                .layer(ClaimCheck::new(|claims: Option<Claims>| {
                    if let Some(claims) = claims {
                        claims.has_scope("superadmin") || claims.has_scope("admin")
                    } else {
                        false
                    }
                })),
        )
    }

    fn migrations(&mut self) -> Option<Vec<Box<dyn mtapp::Migration>>> {
        include_migrations_dir!("./migrations")
    }

    fn public_openapi(&mut self, _: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(PublicSessionOpenApi::openapi())
    }

    fn internal_openapi(&mut self, _: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(InternalSessionOpenApi::openapi())
    }
}
