use std::collections::HashSet;

use schemer::AsyncAdapter;
use sqlx::{
    postgres::PgArguments,
    types::chrono::{DateTime, Local},
    Arguments, Executor, FromRow, PgPool,
};

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
}

impl PgAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_migration_table(&self) -> Result<(), sqlx::Error> {
        self.pool.acquire().await?.execute(CREATE_TABLE).await?;
        Ok(())
    }

    pub async fn applied_migrations_data(&self) -> Result<Vec<AppliedMigration>, sqlx::Error> {
        let res = self
            .pool
            .acquire()
            .await?
            .fetch_all(SELECT_MIGRATIONS)
            .await?;
        Ok(res
            .iter()
            .map(|r| AppliedMigration::from_row(r).expect("Migration table definition is wrong"))
            .collect())
    }
}

#[async_trait::async_trait]
impl AsyncAdapter<MigrationId> for PgAdapter {
    type MigrationType = AppMigration;

    type Error = sqlx::Error;

    async fn applied_migrations(&mut self) -> Result<HashSet<MigrationId>, Self::Error> {
        Ok(self
            .applied_migrations_data()
            .await?
            .into_iter()
            .map(|m| MigrationId::new(m.app.clone(), m.name.clone()))
            .collect())
    }

    async fn apply_migration(&mut self, m: &Self::MigrationType) -> Result<(), Self::Error> {
        println!("Applying migration {}::{}...", m.app, m.migration.name());

        let mut args = PgArguments::default();
        args.add(m.name());
        args.add(m.app());
        args.add(m.description());

        let mut trans = self.pool.begin().await?;
        trans.execute((INSERT_MIGRATION, Some(args))).await?;
        trans.execute(m.up().as_ref()).await?;
        trans.commit().await
    }

    async fn revert_migration(&mut self, m: &Self::MigrationType) -> Result<(), Self::Error> {
        println!("Reverting migration {}::{}...", m.app, m.migration.name());

        let mut args = PgArguments::default();
        args.add(m.name());
        args.add(m.app());

        let mut trans = self.pool.begin().await?;
        trans.execute((DELETE_MIGRATION, Some(args))).await?;
        trans.execute(m.down().as_ref()).await?;
        trans.commit().await
    }
}
