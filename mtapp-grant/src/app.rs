use axum::{
    http::Extensions,
    routing::{delete, get},
    Router,
};
use clap::{Arg, ArgAction, ArgMatches, Command};
use mtapp::App;
use mtapp_auth::{ClaimCheck, Claims};
use sqlx::PgPool;

use crate::{admin, commands::manage_grants};

#[derive(Default)]
pub struct GrantApp {}

impl GrantApp {
    pub fn new() -> Self {
        Self::default()
    }
}

#[axum::async_trait(?Send)]
impl App for GrantApp {
    fn name(&self) -> &'static str {
        "mtapp-grant"
    }

    fn internal_routes(&mut self) -> Option<Router> {
        Some(
            Router::new()
                .route("/grants/:user_id", get(admin::list).post(admin::create))
                .route("/grants/:user_id/:scope_id", delete(admin::delete))
                .layer(ClaimCheck::new(|claims: Option<Claims>| {
                    if let Some(claims) = claims {
                        claims.has_scope("superadmin") || claims.has_scope("admin")
                    } else {
                        false
                    }
                })),
        )
    }

    fn clap_def(&self) -> Option<Command> {
        Some(
            Command::default()
                .about("Management commands for Grant app")
                .subcommand(
                    Command::new("modify")
                        .about("Manage a user's grants")
                        .arg(Arg::new("username").action(ArgAction::Set).required(true)),
                )
                .subcommand_required(true),
        )
    }

    async fn clap_run(&mut self, matches: &ArgMatches, ext: &mut Extensions) {
        let pool = ext.get::<PgPool>().unwrap().clone();
        let recv_username = matches
            .subcommand()
            .expect("Subcommand is required")
            .1
            .get_one::<String>("username")
            .expect("Arg is required")
            .clone();
        manage_grants(pool, recv_username).await
    }
}
