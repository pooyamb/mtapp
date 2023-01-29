use axum::{http::Extensions, Router};
use schemer_migration::Migration;

#[axum::async_trait(?Send)]
pub trait App {
    fn name(&self) -> &'static str;

    /// Public routes
    fn public_routes(&mut self) -> Option<Router> {
        None
    }
    /// Internal routes
    fn internal_routes(&mut self) -> Option<Router> {
        None
    }

    /// Migrations
    fn migrations(&mut self) -> Option<Vec<Box<dyn Migration>>> {
        None
    }

    fn clap_def(&self) -> Option<clap::Command> {
        None
    }

    async fn clap_run(&mut self, _matches: &clap::ArgMatches, _ext: &mut Extensions) {}

    /// Runs once per request
    fn data_register(&self, _ext: &mut Extensions) {}

    /// Should be replaced with something more pleasing
    fn _mod_base_router(&self, router: Router) -> Router {
        router
    }
    fn _mod_public_router(&self, router: Router) -> Router {
        router
    }
    fn _mod_internal_router(&self, router: Router) -> Router {
        router
    }
}
