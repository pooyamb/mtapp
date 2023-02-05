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

use crate::filters::{GrantDeleteFilter, GrantLookupFilter};
use crate::schemas::GrantCreate;

#[derive(Debug, FromRow, Serialize, ToSchema)]
#[enum_def]
pub struct Grant {
    pub(crate) id: Uuid,
    pub(crate) user_id: Uuid,
    pub(crate) scope_id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Iden)]
struct Grants;

impl Grant {
    pub async fn count<'a, E>(
        filters: &QueryFilter<GrantLookupFilter>,
        con: E,
    ) -> Result<i64, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let mut q = Query::select()
            .expr(Expr::asterisk().count())
            .from(Grants)
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
        filters: &QueryFilter<GrantLookupFilter>,
        con: E,
    ) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let (sql, args) = Query::select()
            .expr(Expr::asterisk())
            .from(Grants)
            .to_owned()
            .apply_filters(filters)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub(crate) async fn find_for_user<'a, E>(user_id: Uuid, con: E) -> Result<Vec<String>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        Ok(sqlx::query!(
            "SELECT s.name as scope_name \
                        FROM scopes s INNER JOIN grants g ON g.scope_id = s.id \
                        WHERE g.user_id=$1",
            user_id
        )
        .fetch_all(con)
        .await?
        .into_iter()
        .map(|r| r.scope_name)
        .collect())
    }

    pub async fn create<'a, E>(grant: GrantCreate, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let id = Uuid::new_v4();
        sqlx::query_as!(
            Self,
            "INSERT INTO grants (id, user_id, scope_id) VALUES ($1, $2, $3) RETURNING *",
            id,
            grant.user_id,
            grant.scope_id
        )
        .fetch_one(con)
        .await
    }

    pub async fn delete_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "DELETE FROM grants WHERE id=$1 RETURNING *", id)
            .fetch_one(con)
            .await
    }

    pub async fn delete_by_ids<'a, E>(user_id: Uuid, scope_id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(
            Self,
            "DELETE FROM grants WHERE user_id=$1 AND scope_id=$2 RETURNING *",
            user_id,
            scope_id
        )
        .fetch_one(con)
        .await
    }

    pub async fn delete<'a, E>(filters: &GrantDeleteFilter, con: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        if filters.is_empty() {
            return Err(Error::RowNotFound);
        }

        let (sql, args) = Query::delete()
            .from_table(Grants)
            .to_owned()
            .apply_conds(filters)
            .returning(Query::returning().expr(Expr::asterisk()))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }
}
