use std::{
    any::TypeId,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    task::{Context, Poll},
};

use actix_storage::Storage;
use axum::{
    http::{Extensions, Request},
    routing::IntoMakeService,
    Router,
};
use clap::arg;
use indexmap::IndexMap;
use sqlx::PgPool;
use tower::Service;

use crate::app::App;

pub struct Reactor<D, S> {
    name: &'static str,
    map: IndexMap<TypeId, (&'static str, Box<dyn App + Send + Sync>)>,

    db: D,
    storage: S,
}

impl Reactor<(), ()> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            map: IndexMap::new(),
            db: (),
            storage: (),
        }
    }

    pub fn mount_on<A>(mut self, path: &'static str, app: A) -> Self
    where
        A: App + Send + Sync + 'static,
    {
        self.map.insert(TypeId::of::<A>(), (path, Box::new(app)));
        self
    }
}

impl<D, S> Reactor<D, S> {
    pub fn db(self, db: PgPool) -> Reactor<PgPool, S> {
        Reactor {
            name: self.name,
            map: self.map,
            db,
            storage: self.storage,
        }
    }

    pub fn storage(self, storage: Storage) -> Reactor<D, Storage> {
        Reactor {
            name: self.name,
            map: self.map,
            db: self.db,
            storage,
        }
    }
}

impl Reactor<PgPool, Storage> {
    pub async fn run(mut self) {
        let mut clap_app = clap::Command::new(self.name)
            .arg(arg!(--host <HOST> "Run on host"))
            .arg(arg!(--port <PORT> "Run on port"));

        clap_app =
            clap_app.subcommand(clap::Command::new("migrate").about("Apply all the migrations"));

        for (_, app) in self.map.values() {
            if let Some(cmd) = app.clap_def() {
                clap_app = clap_app.subcommand(cmd.name(app.name()));
            }
        }

        let m = clap_app.get_matches();

        if let Some((name, sm)) = m.subcommand() {
            let mut ext = Extensions::new();
            self.register_states(&mut ext);

            match name {
                "migrate" => {
                    crate::migration::run_migrations(
                        self.db,
                        self.map.values_mut().map(|r| &mut r.1),
                    )
                    .await;
                }
                _ => {
                    for (_, app) in self.map.values_mut() {
                        if app.name() == name {
                            app.clap_run(sm, &mut ext).await
                        }
                    }
                }
            }
        } else {
            let host: IpAddr = m
                .get_one("host")
                .cloned()
                .and_then(|v: &String| v.parse().ok())
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
            let port: u16 = m
                .get_one("port")
                .and_then(|v: &String| v.parse().ok())
                .unwrap_or(3000);
            // run the web server
            axum::Server::bind(&SocketAddr::new(host, port))
                .serve(self.into_make_service())
                .await
                .expect("Failed to start the server");
        }
    }
}

impl Reactor<PgPool, Storage> {
    pub fn into_make_service(mut self) -> IntoMakeService<Router> {
        let public = self.public_router();
        let internal = self.internal_router();

        let mut router = Router::new();
        router = router.merge(Router::new().nest("/public", public));
        router = router.merge(Router::new().nest("/internal", internal));

        for (_, app) in self.map.values_mut().rev() {
            router = app._mod_base_router(router);
        }

        router
            .layer(ReactorLayer(Arc::new(self)))
            .into_make_service()
    }

    fn public_router(&mut self) -> Router {
        let mut router = Router::new();
        for (path, app) in self.map.values_mut() {
            if let Some(routes) = app.public_routes() {
                router = router.merge(Router::new().nest(path, routes));
            }
        }
        for (_, app) in self.map.values_mut().rev() {
            router = app._mod_public_router(router);
        }

        router
    }

    fn internal_router(&mut self) -> Router {
        let mut router = Router::new();
        for (path, app) in self.map.values_mut() {
            if let Some(routes) = app.internal_routes() {
                router = router.merge(Router::new().nest(path, routes));
            }
        }
        for (_, app) in self.map.values_mut().rev() {
            router = app._mod_internal_router(router);
        }

        router
    }

    fn register_states(&self, ext: &mut Extensions) {
        for (_, app) in self.map.values() {
            app.data_register(ext)
        }

        ext.insert(self.storage.clone());
        ext.insert(self.db.clone());
    }
}

#[derive(Clone)]
struct ReactorLayer(Arc<Reactor<PgPool, Storage>>);

impl<S> tower::Layer<S> for ReactorLayer {
    type Service = ReactorLayerRegister<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ReactorLayerRegister {
            inner,
            reactor: self.0.clone(),
        }
    }
}

#[derive(Clone)]
struct ReactorLayerRegister<S> {
    inner: S,
    reactor: Arc<Reactor<PgPool, Storage>>,
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
        self.reactor.register_states(&mut req.extensions_mut());
        self.inner.call(req)
    }
}
