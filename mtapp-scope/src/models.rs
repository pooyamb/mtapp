use sea_query::{enum_def, Iden};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use seaqs::{ApplyConds, ApplyFilters, QueryFilter};
use serde::Serialize;
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::{Error, Executor, FromRow, Postgres, Row};
use utoipa::ToSchema;

use crate::filters::{ScopeDeleteFilter, ScopeLookupFilter};

#[derive(Debug, FromRow, Serialize, ToSchema)]
#[enum_def]
pub struct Scope {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Iden)]
struct Scopes;

impl Scope {
    pub async fn count<'a, E>(
        filters: &QueryFilter<ScopeLookupFilter<'_>>,
        con: E,
    ) -> Result<i64, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let mut q = Query::select()
            .expr(Expr::asterisk().count())
            .from(Scopes)
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

    pub async fn find<'a, E>(
        filters: &QueryFilter<ScopeLookupFilter<'_>>,
        con: E,
    ) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let (sql, args) = Query::select()
            .expr(Expr::asterisk())
            .from(Scopes)
            .to_owned()
            .apply_filters(filters)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub async fn get_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM scopes WHERE id=$1", id)
            .fetch_one(con)
            .await
    }

    pub async fn get_by_name<'a, E>(name: &str, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM scopes WHERE name=$1", name)
            .fetch_one(con)
            .await
    }

    pub async fn create<'a, E>(name: String, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Self,
            "INSERT INTO scopes (id, name) VALUES ($1, $2) RETURNING *",
            id,
            name
        )
        .fetch_one(con)
        .await
    }

    pub async fn delete<'a, E>(filters: &ScopeDeleteFilter, con: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        if filters.is_empty() {
            return Err(Error::RowNotFound);
        }

        let (sql, args) = Query::delete()
            .from_table(Scopes)
            .to_owned()
            .apply_conds(filters)
            .returning(Query::returning().expr(Expr::asterisk()))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub async fn delete_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "DELETE FROM scopes WHERE id=$1 RETURNING *", id)
            .fetch_one(con)
            .await
    }

    pub async fn delete_by_name<'a, E>(name: &str, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "DELETE FROM scopes WHERE name=$1 RETURNING *", name)
            .fetch_one(con)
            .await
    }
}
