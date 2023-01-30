use std::{env, str::FromStr};

use actix_storage::Storage;
use actix_storage_hashmap::HashMapBackend;
use mtapp::Reactor;
use mtapp_auth::AuthApp;
use mtapp_grant::{GrantApp, Provider as GP};
use mtapp_scope::ScopeApp;
use mtapp_session::{Provider as SP, SessionApp};
use mtapp_user::{Provider as UP, UserApp};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let db = get_db().await;
    let storage = get_storage();
    let secret = env::var("APP_SECRET").unwrap();

    let auth_app = AuthApp::<UP, SP, GP>::new(secret);
    let scope_app = ScopeApp::new();
    let user_app = UserApp::new();
    let grant_app = GrantApp::new();
    let session_app = SessionApp::new();

    let app = Reactor::new("myapp")
        .mount_on("/auth", auth_app)
        .mount_on("/scopes", scope_app)
        .mount_on("/users", user_app)
        .mount_on("/grants", grant_app)
        .mount_on("/sessions", session_app)
        .storage(storage)
        .db(db);

    app.run().await
}

async fn get_db() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be in env");

    PgPoolOptions::new()
        .connect_with(
            PgConnectOptions::from_str(&db_url)
                .expect("Invalid DATABASE_URL provided")
                .disable_statement_logging()
                .clone(),
        )
        .await
        .expect("Database connection failed")
}

fn get_storage() -> Storage {
    Storage::build()
        .store(HashMapBackend::start_default())
        .finish()
}
