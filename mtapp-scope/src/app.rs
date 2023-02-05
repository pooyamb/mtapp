use axum::{routing::get, Router};
use mtapp::{include_migrations_dir, App};
use mtapp_auth::{ClaimCheck, Claims};
use utoipa::OpenApi;

use crate::{admin, openapi::InternalScopeOpenApi};

#[derive(Default)]
pub struct ScopeApp {}

impl ScopeApp {
    pub fn new() -> Self {
        ScopeApp {}
    }
}

impl App for ScopeApp {
    fn name(&self) -> &'static str {
        "mtapp-scope"
    }

    fn internal_routes(&mut self) -> Option<Router> {
        Some(
            Router::new()
                .route(
                    "/",
                    get(admin::list)
                        .post(admin::create)
                        .delete(admin::batch_delete),
                )
                .route("/:scope_id", get(admin::get).delete(admin::delete))
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

    fn internal_openapi(&mut self, _: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(InternalScopeOpenApi::openapi())
    }
}
