use std::{borrow::Cow, collections::HashSet, env, fs, path::Path, str::FromStr};

use cargo_toml::Manifest;
use clap::{Arg, Command};
use schemer_migration::{Migration, MigrationId};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};

mod command;

#[tokio::main]
async fn main() {
    let m = get_command().get_matches();

    let (name, m) = m
        .subcommand()
        .expect("It can't happen as we have subcommand_required set to true");

    let db = get_db().await;
    let apps = get_apps();

    let mut migrations = Vec::new();
    for app in apps {
        if Path::new(&app.migrations_dir).exists() {
            if let Some(app_migrations) = collect_migrations(app.name, &app.migrations_dir) {
                migrations.extend(app_migrations);
            }
        }
    }

    match name {
        "run" => command::run_migrations(m, db, migrations.into_iter()).await,
        "list" => command::list_migrations(db, migrations.into_iter()).await,
        "revert" => command::revert_migrations(m, db, migrations.into_iter()).await,
        _ => {
            unreachable!()
        }
    }
}

struct App {
    name: String,
    migrations_dir: String,
}

#[derive(Debug, Deserialize)]
pub struct MigrationData {
    dependencies: Vec<String>,
    description: String,

    #[serde(skip_deserializing)]
    name: String,

    #[serde(skip_deserializing)]
    up: String,
    #[serde(skip_deserializing)]
    down: String,
}

impl Migration for MigrationData {
    fn dependencies(&self) -> HashSet<MigrationId> {
        HashSet::from_iter(
            self.dependencies
                .iter()
                .map(|v| MigrationId::try_from(v).expect(&format!("Invalid dependency found {v}"))),
        )
    }

    fn name(&self) -> Cow<'static, str> {
        self.name.clone().into()
    }

    fn description(&self) -> &'static str {
        let description = self.description.clone().into_boxed_str();
        Box::leak(description)
    }

    fn up(&self) -> Cow<'static, str> {
        self.up.clone().into()
    }

    fn down(&self) -> Cow<'static, str> {
        self.down.clone().into()
    }
}

fn collect_migrations(app: String, path: &str) -> Option<Vec<(Cow<'static, str>, MigrationData)>> {
    let mut migrations = Vec::new();
    for folder in fs::read_dir(path).ok()?.into_iter() {
        if let Ok(folder) = folder {
            let name = if let Ok(type_) = folder.file_type() {
                if !type_.is_dir() {
                    continue;
                }
                folder.file_name()
            } else {
                continue;
            };

            // TODO: improve this part to return errors and give context
            // Read the meta json
            let mut json_path = folder.path();
            json_path.push(".meta.json");
            let meta_data =
                fs::read(json_path).expect("Migration folder should have .meta.json file.");
            let mut migration: MigrationData =
                serde_json::from_slice(&meta_data).expect(".meta.json has invalid format.");

            let mut up_path = folder.path();
            up_path.push("up.sql");
            let up = fs::read(up_path).expect("Migration folder should have up.sql file.");
            migration.up = String::from_utf8(up).expect("up.sql is not valid utf-8");

            let mut down_path = folder.path();
            down_path.push("down.sql");
            let down = fs::read(down_path).expect("Migration folder should have down.sql file.");
            migration.down = String::from_utf8(down).expect("down.sql is not valid utf-8");

            migration.name = String::from(name.to_str().expect("Name is not valid utf-8"));

            migrations.push((Cow::Owned(app.clone()), migration));
        }
    }
    Some(migrations)
}

async fn get_db() -> PgPool {
    dotenvy::dotenv().ok();
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

fn get_apps() -> Vec<App> {
    let members = Manifest::from_path("Cargo.toml")
        .expect("Should be called inside a cargo project")
        .workspace
        .expect("Should be called inside workspace's root")
        .members;

    let mut apps = Vec::new();
    for member in members {
        if member == "." {
            continue;
        }
        if let Ok(cargo) = Manifest::from_path(format!("./{}/Cargo.toml", member)) {
            if let Some(package) = cargo.package {
                apps.push(App {
                    name: package.name,
                    migrations_dir: format!("./{}/migrations", member),
                });
                continue;
            }
        }
        println!("Ignoring the workspace member {}", member)
    }
    apps
}

pub fn get_command() -> Command {
    Command::new("migrate")
        .about("Manage the database migrations")
        .subcommand(
            Command::new("run")
                .about("Run migrations")
                .arg(
                    Arg::new("name").short('n').long("name").help(
                        "Run to this migration, format should be {app_name}:{migration_name}",
                    ),
                )
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("Run to this migration, value should be valid uuid"),
                ),
        )
        .subcommand(Command::new("list").about("List migrations"))
        .subcommand(
            Command::new("revert")
                .about("Revert migrations")
                .arg(
                    Arg::new("name").short('n').long("name").help(
                        "Run to this migration, format should be {app_name}:{migration_name}",
                    ),
                )
                .arg(
                    Arg::new("id")
                        .short('i')
                        .long("id")
                        .help("Run to this migration, value should be valid uuid"),
                ),
        )
        .subcommand_required(true)
}
