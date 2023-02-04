use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use basteh::Storage;
use basteh_memory::MemoryBackend;
use clap::{arg, Command};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};

use mtapp::Reactor;
use mtapp_auth::AuthApp;
use mtapp_grant::{GrantApp, Provider as GP};
use mtapp_scope::ScopeApp;
use mtapp_session::{Provider as SP, SessionApp};
use mtapp_user::{Provider as UP, UserApp};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be in env");
    let secret = env::var("APP_SECRET").expect("APP_SECRET should be in env");
    let host = env::var("APP_HOST").ok();
    let port = env::var("APP_PORT").ok();

    let db = get_db(&db_url).await;
    let storage = get_storage();

    let auth_app = AuthApp::<UP, SP, GP>::new(secret);
    let scope_app = ScopeApp::new();
    let user_app = UserApp::new();
    let grant_app = GrantApp::new();
    let session_app = SessionApp::new();

    let mut app = Reactor::new()
        .public_path("/api/dev")
        .internal_path("/api/internals")
        .mount_on("/auth", auth_app)
        .mount_on("/scopes", scope_app)
        .mount_on("/users", user_app)
        .mount_on("/grants", grant_app)
        .mount_on("/sessions", session_app)
        .storage(storage)
        .db(db);

    let m = get_clap_defs(&app).get_matches();

    match m.subcommand() {
        Some(("migrate", _)) => {
            app.run_migrations().await;
        }
        Some((cmd, args)) => {
            app.run_command(cmd, args).await;
        }
        None => {
            let host: IpAddr = m
                .get_one("host")
                .cloned()
                .or(host)
                .and_then(|v| v.parse().ok())
                .unwrap_or(Ipv4Addr::new(127, 0, 0, 1).into());

            let port: u16 = m
                .get_one("port")
                .cloned()
                .or(port)
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000);

            let api_docs = app.public_api_docs();
            let internal_api_docs = app.internal_api_docs();
            let mut router = app.into_router();
            router = router
                .merge(SwaggerUi::new("/api/dev/docs").url("/api/dev/api-docs.json", api_docs))
                .merge(
                    SwaggerUi::new("/api/internals/docs")
                        .url("/api/internals/api-docs.json", internal_api_docs),
                );

            log::info!("Running web server on: http://{}:{}", host, port);

            axum::Server::bind(&SocketAddr::new(host, port))
                .serve(router.into_make_service())
                .await
                .expect("Failed to start the server");
        }
    }
}

fn get_clap_defs<D, S>(app: &Reactor<D, S>) -> Command {
    // Run the web server when there is no commands
    let mut clap_app = Command::new("myapp")
        .arg(arg!(--host <HOST> "Run on host"))
        .arg(arg!(--port <PORT> "Run on port"));

    // Register the migrate command
    clap_app = clap_app.subcommand(clap::Command::new("migrate").about("Apply all the migrations"));

    // Register all subcommands from app
    clap_app.subcommands(app.clap_defs())
}

async fn get_db(db_url: &str) -> PgPool {
    PgPoolOptions::new()
        .connect_with(
            PgConnectOptions::from_str(db_url)
                .expect("Invalid DATABASE_URL provided")
                .disable_statement_logging()
                .clone(),
        )
        .await
        .expect("Database connection failed")
}

fn get_storage() -> Storage {
    Storage::build()
        .store(MemoryBackend::start_default())
        .finish()
}
