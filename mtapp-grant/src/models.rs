use serde::Serialize;
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::{Error, Executor, FromRow, Postgres};

#[derive(Serialize, FromRow)]
pub struct Grant {
    pub(crate) user_id: Uuid,
    pub(crate) scope_id: Uuid,
    pub(crate) scope_name: String,
    pub(crate) created_at: DateTime<Utc>,
}

impl Grant {
    pub(crate) async fn get_grants<'a, E>(user_id: Uuid, con: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "SELECT g.user_id, g.scope_id, s.name as scope_name, g.created_at \
                FROM scopes s INNER JOIN grants g ON g.scope_id = s.id \
                WHERE g.user_id=$1",
            user_id
        )
        .fetch_all(con)
        .await
    }

    pub(crate) async fn add_grant<'a, E>(
        user_id: Uuid,
        scope_name: &str,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Self,
            "INSERT INTO grants \
                (id, user_id, scope_id) \
                VALUES ($1, $2, (SELECT id FROM scopes WHERE scopes.name = $3)) \
                RETURNING user_id, scope_id, $3 as \"scope_name!\", created_at",
            id,
            user_id,
            scope_name,
        )
        .fetch_one(con)
        .await
    }

    pub(crate) async fn del_grant<'a, E>(
        user_id: Uuid,
        scope_name: &str,
        con: E,
    ) -> Result<(), Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "DELETE FROM grants WHERE user_id=$1 AND scope_id=(SELECT id FROM scopes WHERE scopes.name = $2)",
            user_id,
            scope_name,
        )
        .execute(con)
        .await?;
        Ok(())
    }
}
