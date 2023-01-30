use std::{borrow::Cow, collections::HashMap};

use clap::ArgMatches;
use cli_table::{print_stdout, Cell, Style, Table};
use schemer::Migrator;
use sqlx::PgPool;

use schemer_migration::{AppMigration, Migration, MigrationId, PgAdapter};

/// TODO: should be improved to give an error instead of panics.
pub async fn run_migrations<T: Migration + 'static>(
    m: &ArgMatches,
    db: PgPool,
    migrations: impl Iterator<Item = (Cow<'static, str>, T)>,
) {
    let adapter = PgAdapter::new(db.clone());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");

    let mut migrator = Migrator::new_async(adapter);
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
        migrator.up_async(Some(id.clone())).await.unwrap();
    } else {
        migrator.up_async(None).await.unwrap();
    }
}

pub async fn list_migrations<T: Migration + 'static>(
    db: PgPool,
    migrations: impl Iterator<Item = (Cow<'static, str>, T)>,
) {
    let adapter = PgAdapter::new(db.clone());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");

    let applied_migrations = adapter
        .applied_migrations_data()
        .await
        .expect("Can't happen, method returns Ok");

    let mut table = Vec::new();
    for migration in applied_migrations.iter() {
        table.push(vec![
            "Yes".cell(),
            (&migration.app).cell(),
            (&migration.name).cell(),
            (&migration.description).cell(),
            migration.created_at.to_rfc2822().cell(),
        ])
    }

    let mut notaplied_migrations = Vec::new();
    notaplied_migrations.extend(
        migrations
            .into_iter()
            .filter(|migration| {
                !applied_migrations
                    .iter()
                    .any(|m| migration.0 == m.app && migration.1.name() == m.name)
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
    let adapter = PgAdapter::new(db.clone());

    adapter
        .create_migration_table()
        .await
        .expect("Couldn't create the migration table");

    let mut migrator = Migrator::new_async(adapter);
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
        migrator.down_async(Some(id.clone())).await.unwrap();
    } else {
        migrator.down_async(None).await.unwrap();
    }
}
