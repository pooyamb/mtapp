use sea_query::{enum_def, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use seaqs::{ApplyConds, ApplyFilters, QueryFilter};
use serde::Serialize;
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::{Error, Executor, FromRow, Postgres, Row};

use crate::filters::{SessionDeleteFilter, SessionLookupFilter};

#[derive(Serialize, FromRow)]
#[enum_def]
pub struct Session {
    pub(crate) id: Uuid,
    pub(crate) user_id: Uuid,
    pub(crate) ip: String,
    pub(crate) user_agent: String,
    #[serde(skip)]
    pub(crate) jti: Uuid,
    #[serde(skip)]
    pub(crate) refresh_token: Uuid,
    last_access_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

#[derive(Iden)]
struct Sessions;

impl Session {
    pub async fn count<'a, E>(
        filters: &QueryFilter<SessionLookupFilter>,
        con: E,
    ) -> Result<i64, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let mut q = Query::select()
            .expr(Expr::asterisk().count())
            .from(Sessions)
            .to_owned();

        if let Some(filter) = &filters.filter {
            q = q.apply_conds(filter).to_owned();
        };

        let (sql, args) = q.build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, args)
            .try_map(|row: sqlx::postgres::PgRow| {
                let count = row.try_get_unchecked::<Option<i64>, _>(0usize)?;
                Ok(count)
            })
            .fetch_one(con)
            .await
            .map(|v| v.unwrap_or(0))
    }

    pub(crate) async fn count_by_user<'a, E>(user_id: Uuid, con: E) -> Result<i64, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(
            sqlx::query!("SELECT COUNT(*) FROM sessions WHERE user_id = $1", user_id)
                .fetch_one(con)
                .await?
                .count
                .unwrap_or(0),
        )
    }

    pub async fn find<'a, E>(
        filters: &QueryFilter<SessionLookupFilter>,
        con: E,
    ) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let (sql, args) = Query::select()
            .expr(Expr::asterisk())
            .from(Sessions)
            .to_owned()
            .apply_filters(filters)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub(crate) async fn find_by_user<'a, E>(user_id: Uuid, con: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM sessions WHERE user_id=$1", user_id)
            .fetch_all(con)
            .await
    }

    pub(crate) async fn get_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM sessions WHERE id = $1", id)
            .fetch_one(con)
            .await
    }

    pub(crate) async fn get_by_id_for_user<'a, E>(
        user_id: Uuid,
        id: Uuid,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "SELECT * FROM sessions WHERE user_id = $1 AND id = $2",
            user_id,
            id
        )
        .fetch_one(con)
        .await
    }

    pub(crate) async fn get_by_jti<'a, E>(jti: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM sessions WHERE jti = $1", jti)
            .fetch_one(con)
            .await
    }

    pub(crate) async fn get_by_refresh_token<'a, E>(
        refresh_token: Uuid,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "SELECT * FROM sessions WHERE refresh_token = $1",
            refresh_token
        )
        .fetch_one(con)
        .await
    }

    pub(crate) async fn create<'a, E>(
        user_id: Uuid,
        ip: String,
        user_agent: String,
        jti: Uuid,
        refresh_token: Uuid,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "INSERT INTO sessions (id, user_id, ip, user_agent, jti, refresh_token) VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *",
            Uuid::new_v4(),
            user_id,
            ip,
            user_agent,
            jti,
            refresh_token
        )
        .fetch_one(con)
        .await
    }

    pub(crate) async fn set_jti<'a, E>(
        refresh_token: Uuid,
        jti: Uuid,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "UPDATE sessions SET jti=$1 WHERE refresh_token=$2
                RETURNING *",
            jti,
            refresh_token
        )
        .fetch_one(con)
        .await
    }

    pub async fn delete<'a, E>(filters: &SessionDeleteFilter, con: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        if filters.is_empty() {
            return Err(Error::RowNotFound);
        }

        let (sql, args) = Query::delete()
            .from_table(Sessions)
            .to_owned()
            .apply_conds(filters)
            .returning(Query::returning().expr(Expr::asterisk()))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub(crate) async fn delete_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "DELETE FROM sessions WHERE id = $1 RETURNING *", id)
            .fetch_one(con)
            .await
    }

    pub(crate) async fn delete_by_id_for_user<'a, E>(
        user_id: Uuid,
        id: Uuid,
        con: E,
    ) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "DELETE FROM sessions WHERE user_id = $1 AND id = $2 RETURNING *",
            user_id,
            id
        )
        .fetch_one(con)
        .await
    }

    pub(crate) async fn delete_by_jti<'a, E>(jti: Uuid, con: E) -> Result<Uuid, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query!("DELETE FROM sessions WHERE jti = $1 RETURNING id", jti)
            .fetch_one(con)
            .await
            .map(|r| r.id)
    }
}
