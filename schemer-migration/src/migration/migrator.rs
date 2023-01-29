use std::{collections::HashSet, thread};

use schemer::Adapter;
use sqlx::{
    postgres::PgArguments,
    types::chrono::{DateTime, Local},
    Arguments, Executor, FromRow, PgPool,
};
use tokio::runtime::Handle;

use super::{def::AppMigration, id::MigrationId};

const CREATE_TABLE: &'static str = r#"
-- DROP TABLE IF EXISTS _schemer_migrations_;
CREATE TABLE IF NOT EXISTS _schemer_migrations_ (
    name varchar(255) NOT NULL,
    app varchar(255) NOT NULL,
    description text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);
"#;

const SELECT_MIGRATIONS: &'static str = r#"
SELECT name, app, description, created_at FROM _schemer_migrations_;
"#;

const INSERT_MIGRATION: &'static str = r#"
INSERT INTO _schemer_migrations_ (name, app, description) VALUES ($1, $2, $3);
"#;

const DELETE_MIGRATION: &'static str = r#"
DELETE FROM _schemer_migrations_ WHERE name = $1 and app = $2;
"#;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct AppliedMigration {
    pub name: String,
    pub app: String,
    pub description: String,
    pub created_at: DateTime<Local>,
}

pub struct PgAdapter {
    pool: PgPool,
    handle: Handle,
    applied: Vec<AppliedMigration>,
}

impl PgAdapter {
    pub fn new(pool: PgPool, handle: Handle) -> Self {
        Self {
            pool,
            handle,
            applied: Vec::new(),
        }
    }

    pub async fn create_migration_table(&self) -> Result<(), sqlx::Error> {
        self.pool.acquire().await?.execute(CREATE_TABLE).await?;
        Ok(())
    }

    pub async fn load(&mut self) -> Result<(), sqlx::Error> {
        let res = self
            .pool
            .acquire()
            .await?
            .fetch_all(SELECT_MIGRATIONS)
            .await?;
        self.applied = res
            .iter()
            .map(|r| AppliedMigration::from_row(r).expect("Migration table definition is wrong"))
            .collect::<Vec<_>>();
        Ok(())
    }

    pub fn into_applied_migrations(self) -> Vec<AppliedMigration> {
        self.applied
    }
}

impl Adapter<MigrationId> for PgAdapter {
    type MigrationType = AppMigration;

    type Error = sqlx::Error;

    fn applied_migrations(&mut self) -> Result<HashSet<MigrationId>, Self::Error> {
        Ok(self
            .applied
            .iter()
            .map(|m| MigrationId::new(m.app.clone(), m.name.clone()))
            .collect())
    }

    fn apply_migration(&mut self, m: &Self::MigrationType) -> Result<(), Self::Error> {
        let handle = self.handle.clone();
        let pool = self.pool.clone();
        let migration = m.clone();

        println!("Applying migration {}::{}...", m.app, m.migration.name());

        // TODO: Don't spawn threads for each migration
        let r = thread::spawn(move || {
            handle.block_on(async {
                let mut args = PgArguments::default();
                args.add(migration.name());
                args.add(migration.app());
                args.add(migration.description());

                let mut trans = pool.begin().await?;
                trans.execute((INSERT_MIGRATION, Some(args))).await?;
                trans.execute(migration.up().as_ref()).await?;
                trans.commit().await
            })
        })
        .join()
        .expect("Coudn't spawn a thread");

        r
    }

    fn revert_migration(&mut self, m: &Self::MigrationType) -> Result<(), Self::Error> {
        let handle = self.handle.clone();
        let pool = self.pool.clone();
        let migration = m.clone();

        println!("Reverting migration {}::{}...", m.app, m.migration.name());

        // TODO: Don't spawn threads for each migration
        let r = thread::spawn(move || {
            handle.block_on(async {
                let mut args = PgArguments::default();
                args.add(migration.name());
                args.add(migration.app());

                let mut trans = pool.begin().await?;
                trans.execute((DELETE_MIGRATION, Some(args))).await?;
                trans.execute(migration.down().as_ref()).await?;
                trans.commit().await
            })
        })
        .join()
        .expect("Coudn't spawn a thread");

        r
    }
}
