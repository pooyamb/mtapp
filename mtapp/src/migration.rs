use std::{borrow::Cow, collections::HashMap};

use sqlx::PgPool;

use schemer_migration::{AppMigration, Migration, Migrator, PgAdapter};

use crate::App;

pub(crate) fn get_local_migrations() -> Option<Vec<Box<dyn Migration>>> {
    crate::include_migrations_dir!("./migrations" => "crate")
}

pub(crate) async fn run_migrations(
    db: PgPool,
    apps: impl Iterator<Item = &mut Box<dyn App + Send + Sync>>,
) {
    let adapter = PgAdapter::new(db.clone());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");

    let mut migrator = Migrator::new_async(adapter);
    let mut id_name_map = HashMap::new();
    let mut migrations = Vec::new();

    for m in get_local_migrations()
        .unwrap()
        .into_iter()
        .map(|migration| AppMigration::new(Cow::Borrowed("mtapp"), migration))
    {
        id_name_map.insert((m.app(), m.name()), m.id());
        migrations.push(m.clone());
    }

    for app in apps {
        if let Some(app_migrations) = app.migrations() {
            for m in app_migrations
                .into_iter()
                .map(|migration| AppMigration::new(Cow::Borrowed(app.name()), migration))
            {
                id_name_map.insert((m.app(), m.name()), m.id());
                migrations.push(m.clone());
            }
        }
    }

    migrator.register_multiple(migrations.into_iter()).unwrap();

    migrator.up_async(None).await.unwrap();
}
