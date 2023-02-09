use axum::{
    http::Extensions,
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use clap::{Arg, Command};
use mtapp::{include_migrations_dir, App, Configuration, Migration};
use mtapp_auth::{ClaimCheck, Claims};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    admin, commands, handlers,
    middlware::user_ban_check,
    openapi::{InternalUserOpenApi, PublicUserOpenApi},
};

#[derive(Default, Clone)]
pub struct UserApp {}

impl UserApp {
    pub fn new() -> Self {
        sodiumoxide::init().expect("Libsodium init failed");
        UserApp {}
    }
}

#[axum::async_trait(?Send)]
impl App for UserApp {
    fn name(&self) -> &'static str {
        "mtapp-user"
    }

    fn configure(&mut self, cfg: &mut Configuration) {
        cfg.base_router(|router| router.layer(from_fn(user_ban_check)));
    }

    fn public_routes(&mut self) -> Option<Router> {
        Some(
            Router::new().route("/", post(handlers::signup)).merge(
                Router::new()
                    .route("/me", get(handlers::get_me).post(handlers::update))
                    .layer(ClaimCheck::new(|claims: Option<Claims>| claims.is_some())),
            ),
        )
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
                .route(
                    "/:user_id",
                    get(admin::get).post(admin::update).delete(admin::delete),
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

    fn migrations(&mut self) -> Option<Vec<Box<dyn Migration>>> {
        include_migrations_dir!("./migrations")
    }

    fn clap_def(&self) -> Option<clap::Command> {
        Some(
            Command::default()
                .about("Management commands for User app")
                .subcommand(
                    Command::new("create_user")
                        .about("Create a new superuser account")
                        .arg(Arg::new("username").short('u').long("username"))
                        .arg(Arg::new("password").short('p').long("password")),
                )
                .subcommand_required(true),
        )
    }

    async fn clap_run(&mut self, matches: &clap::ArgMatches, ext: &Extensions) {
        let pool = ext
            .get::<PgPool>()
            .expect("Inserted into extensions by reactor")
            .clone();

        match matches.subcommand() {
            Some(("create_user", sub_m)) => {
                let username = sub_m.get_one::<String>("username").cloned();
                let password = sub_m.get_one::<String>("password").cloned();
                commands::create_user(pool, username, password).await;
            }
            _ => {
                // Subcommand is required in clap definition
                unreachable!()
            }
        }
    }

    fn public_openapi(&mut self, _: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(PublicUserOpenApi::openapi())
    }

    fn internal_openapi(&mut self, _: &str) -> Option<utoipa::openapi::OpenApi> {
        Some(InternalUserOpenApi::openapi())
    }
}
