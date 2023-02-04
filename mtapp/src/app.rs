use axum::{http::Extensions, Router};
use schemer_migration::Migration;
use utoipa::openapi::OpenApi;

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

    async fn clap_run(&mut self, _matches: &clap::ArgMatches, _ext: &Extensions) {}

    fn configure(&mut self, _cfg: &mut Configuration) {}

    fn public_openapi(&mut self, _path: &str) -> Option<OpenApi> {
        None
    }

    fn internal_openapi(&mut self, _path: &str) -> Option<OpenApi> {
        None
    }
}

pub struct Configuration {
    global_state: Option<Box<dyn Fn(&mut Extensions) + Send + Sync>>,
    base_router: Option<Box<dyn Fn(Router) -> Router + Send + Sync>>,
    public_router: Option<Box<dyn Fn(Router) -> Router + Send + Sync>>,
    internal_router: Option<Box<dyn Fn(Router) -> Router + Send + Sync>>,
}

impl Configuration {
    pub(crate) fn new() -> Self {
        Self {
            global_state: None,
            base_router: None,
            public_router: None,
            internal_router: None,
        }
    }

    pub fn global_state<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&mut Extensions) + Send + Sync + 'static,
    {
        self.global_state = Some(Box::new(f));
        self
    }

    pub fn base_router<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(Router) -> Router + Send + Sync + 'static,
    {
        self.base_router = Some(Box::new(f));
        self
    }

    pub fn public_router<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(Router) -> Router + Send + Sync + 'static,
    {
        self.public_router = Some(Box::new(f));
        self
    }

    pub fn internal_router<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(Router) -> Router + Send + Sync + 'static,
    {
        self.internal_router = Some(Box::new(f));
        self
    }

    pub(crate) fn into_global_state(self) -> Option<Box<dyn Fn(&mut Extensions) + Send + Sync>> {
        self.global_state
    }

    pub(crate) fn configure_global_state(&self, ext: &mut Extensions) {
        if let Some(f) = &self.global_state {
            f(ext);
        }
    }

    pub(crate) fn configure_base_router(&self, router: Router) -> Router {
        if let Some(f) = &self.base_router {
            f(router)
        } else {
            router
        }
    }

    pub(crate) fn configure_public_router(&self, router: Router) -> Router {
        if let Some(f) = &self.public_router {
            f(router)
        } else {
            router
        }
    }

    pub(crate) fn configure_internal_router(&self, router: Router) -> Router {
        if let Some(f) = &self.internal_router {
            f(router)
        } else {
            router
        }
    }
}
