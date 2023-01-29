use std::{borrow::Cow, collections::HashMap};

use clap::ArgMatches;
use cli_table::{print_stdout, Cell, Style, Table};
use schemer::{Adapter, Migrator};
use sqlx::PgPool;
use tokio::runtime::Handle;

use schemer_migration::{AppMigration, Migration, MigrationId, PgAdapter};

/// TODO: should be improved to give an error instead of panics.
pub async fn run_migrations<T: Migration + 'static>(
    m: &ArgMatches,
    db: PgPool,
    migrations: impl Iterator<Item = (Cow<'static, str>, T)>,
) {
    let mut adapter = PgAdapter::new(db.clone(), Handle::current());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");
    adapter
        .load()
        .await
        .expect("Couldn't load applied migrations.");

    let mut migrator = Migrator::new(adapter);
    let mut id_name_map = HashMap::new();

    let migrations = migrations
        .into_iter()
        .map(|migration| AppMigration::new(migration.0, migration.1))
        .collect::<Vec<_>>();

    for m in migrations.iter() {
        id_name_map.insert((m.app(), m.name()), m.id());
    }
    migrator.register_multiple(migrations.into_iter()).unwrap();

    if let Some(name) = m.get_one::<String>("name") {
        let id = MigrationId::try_from(name).expect("Invalid migration name is given");
        migrator.up(Some(id.clone())).unwrap();
    } else {
        migrator.up(None).unwrap();
    }
}

pub async fn list_migrations<T: Migration + 'static>(
    db: PgPool,
    migrations: impl Iterator<Item = (Cow<'static, str>, T)>,
) {
    let mut adapter = PgAdapter::new(db.clone(), Handle::current());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");
    adapter
        .load()
        .await
        .expect("Couldn't load applied migrations.");

    let applied_ids = adapter
        .applied_migrations()
        .expect("Can't happen, method returns Ok");
    let applied_migrations = adapter.into_applied_migrations();

    let mut table = Vec::new();
    for migration in applied_migrations.into_iter() {
        table.push(vec![
            "Yes".cell(),
            migration.app.cell(),
            migration.name.cell(),
            migration.description.cell(),
            migration.created_at.to_rfc2822().cell(),
        ])
    }

    let mut notaplied_migrations = Vec::new();
    notaplied_migrations.extend(
        migrations
            .into_iter()
            .filter(|migration| {
                !applied_ids.contains(&MigrationId::new(
                    migration.0.to_string(),
                    migration.1.name().to_string(),
                ))
            })
            .map(|migration| AppMigration::new(migration.0, migration.1))
            .collect::<Vec<_>>(),
    );

    for migration in notaplied_migrations.into_iter() {
        table.push(vec![
            "No".cell(),
            migration.app().cell(),
            migration.name().cell(),
            migration.description().cell(),
            "NA".cell(),
        ])
    }

    let table = table
        .table()
        .title(vec![
            "Is up?".cell().bold(true),
            "App".cell().bold(true),
            "Name".cell().bold(true),
            "Description".cell().bold(true),
            "Created at".cell().bold(true),
        ])
        .bold(true);

    print_stdout(table).unwrap();
}

/// TODO: should be improved to give an error instead of panics.
pub async fn revert_migrations<T: Migration + 'static>(
    m: &ArgMatches,
    db: PgPool,
    migrations: impl Iterator<Item = (Cow<'static, str>, T)>,
) {
    let mut adapter = PgAdapter::new(db.clone(), Handle::current());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");
    adapter
        .load()
        .await
        .expect("Couldn't load applied migrations.");

    let mut migrator = Migrator::new(adapter);
    let mut id_name_map = HashMap::new();

    let migrations = migrations
        .into_iter()
        .map(|migration| AppMigration::new(migration.0, migration.1))
        .collect::<Vec<_>>();

    for m in migrations.iter() {
        id_name_map.insert((m.app(), m.name()), m.id());
    }
    migrator.register_multiple(migrations.into_iter()).unwrap();

    if let Some(name) = m.get_one::<String>("name") {
        let id = MigrationId::try_from(name).expect("Invalid migration name is given");
        migrator.down(Some(id.clone())).unwrap();
    } else {
        migrator.down(None).unwrap();
    }
}
