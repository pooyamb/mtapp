use std::{
    borrow::Cow,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    http::{Extensions, Request},
    Router,
};
use basteh::Storage;
use clap::{ArgMatches, Command};
use indexmap::IndexMap;
use sqlx::PgPool;
use tower::Service;

use crate::app::{App, Configuration};

pub struct Reactor<D, S> {
    map: IndexMap<&'static str, Box<dyn App>>,
    cfgs: Vec<Configuration>,

    public_path: Option<Cow<'static, str>>,
    internal_path: Option<Cow<'static, str>>,

    db: D,
    storage: S,
}

impl Reactor<(), ()> {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            cfgs: Vec::new(),

            public_path: None,
            internal_path: None,

            db: (),
            storage: (),
        }
    }
}

impl<D, S> Reactor<D, S> {
    pub fn mount_on<A>(mut self, path: &'static str, app: A) -> Self
    where
        A: App + 'static,
    {
        self.map.insert(path, Box::new(app));
        self
    }

    pub fn public_path(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.public_path = Some(path.into());
        self
    }

    pub fn internal_path(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.internal_path = Some(path.into());
        self
    }

    pub fn db(self, db: PgPool) -> Reactor<PgPool, S> {
        Reactor {
            map: self.map,
            cfgs: self.cfgs,

            public_path: self.public_path,
            internal_path: self.internal_path,

            db,
            storage: self.storage,
        }
    }

    pub fn storage(self, storage: Storage) -> Reactor<D, Storage> {
        Reactor {
            map: self.map,
            cfgs: self.cfgs,

            public_path: self.public_path,
            internal_path: self.internal_path,

            db: self.db,
            storage,
        }
    }

    pub fn clap_defs(&self) -> Vec<Command> {
        let mut commands = Vec::new();
        for app in self.map.values() {
            if let Some(cmd) = app.clap_def() {
                commands.push(cmd.name(app.name()));
            }
        }
        commands
    }
}

impl Reactor<PgPool, Storage> {
    pub async fn run_migrations(&mut self) {
        crate::migration::run_migrations(self.db.clone(), self.map.values_mut()).await;
    }

    pub async fn run_command(mut self, subcommand: &str, args: &ArgMatches) {
        let ext = self.get_extensions();
        for app in self.map.values_mut() {
            if app.name() == subcommand {
                app.clap_run(args, &ext).await
            }
        }
    }

    pub fn get_extensions(&mut self) -> Extensions {
        self.cfgs = self
            .map
            .values_mut()
            .map(|app| {
                let mut cfg = Configuration::new();
                app.configure(&mut cfg);
                cfg
            })
            .collect();

        let mut ext = Extensions::new();
        for cfg in self.cfgs.iter() {
            cfg.configure_global_state(&mut ext)
        }

        ext.insert(self.storage.clone());
        ext.insert(self.db.clone());
        ext
    }

    pub fn into_router(mut self) -> Router {
        self.cfgs = self
            .map
            .values_mut()
            .map(|app| {
                let mut cfg = Configuration::new();
                app.configure(&mut cfg);
                cfg
            })
            .collect();

        let mut router = Router::new();

        if self.public_path.is_some() {
            let sub_router = self.public_router();
            router = router.merge(Router::new().nest(
                &self.public_path.as_ref().expect("Just checked is_some"),
                sub_router,
            ));
        }

        if self.internal_path.is_some() {
            let sub_router = self.internal_router();
            router = router.merge(Router::new().nest(
                &self.internal_path.as_ref().expect("Just checked is_some"),
                sub_router,
            ));
        }

        for cfg in self.cfgs.iter() {
            router = cfg.configure_base_router(router);
        }

        router.layer(ReactorLayer(ReactorLayerInner {
            db: self.db,
            storage: self.storage,
            state_fns: Arc::new(
                self.cfgs
                    .into_iter()
                    .map(|v| v.into_global_state())
                    .flatten()
                    .collect(),
            ),
        }))
    }

    fn public_router(&mut self) -> Router {
        let mut router = Router::new();
        for (path, app) in self.map.iter_mut() {
            if let Some(routes) = app.public_routes() {
                router = router.merge(Router::new().nest(path, routes));
            }
        }
        for cfg in self.cfgs.iter() {
            router = cfg.configure_public_router(router);
        }

        router
    }

    fn internal_router(&mut self) -> Router {
        let mut router = Router::new();
        for (path, app) in self.map.iter_mut() {
            if let Some(routes) = app.internal_routes() {
                router = router.merge(Router::new().nest(path, routes));
            }
        }

        for cfg in self.cfgs.iter() {
            router = cfg.configure_internal_router(router);
        }

        router
    }
}

#[derive(Clone)]
struct ReactorLayerInner {
    db: PgPool,
    storage: Storage,
    state_fns: Arc<Vec<Box<dyn Fn(&mut Extensions) + Send + Sync>>>,
}

#[derive(Clone)]
struct ReactorLayer(ReactorLayerInner);

impl<S> tower::Layer<S> for ReactorLayer {
    type Service = ReactorLayerRegister<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ReactorLayerRegister {
            inner,
            data: self.0.clone(),
        }
    }
}

#[derive(Clone)]
struct ReactorLayerRegister<S> {
    inner: S,
    data: ReactorLayerInner,
}

impl<ResBody, S> Service<Request<ResBody>> for ReactorLayerRegister<S>
where
    S: Service<Request<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ResBody>) -> Self::Future {
        let ext = req.extensions_mut();
        for f in self.data.state_fns.iter() {
            f(ext);
        }

        ext.insert(self.data.storage.clone());
        ext.insert(self.data.db.clone());
        self.inner.call(req)
    }
}
