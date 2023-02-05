use sea_query::{enum_def, Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use seaqs::{ApplyConds, ApplyFilters, QueryFilter};
use serde::Serialize;
use sqlx::types::{
    chrono::{DateTime, Utc},
    Uuid,
};
use sqlx::{Error, Executor, FromRow, Postgres, Row};
use utoipa::ToSchema;

use crate::filters::{UserDeleteFilter, UserLookupFilter};
use crate::helpers;
use crate::schemas::{UserCreate, UserUpdate};

#[derive(Serialize, FromRow, ToSchema)]
#[enum_def]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub last_logged_in_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip_serializing)]
    password: String,
}

#[derive(Iden)]
pub struct Users;

impl Default for User {
    fn default() -> User {
        User {
            id: Uuid::default(),
            username: String::new(),
            password: String::new(),
            email: None,
            last_logged_in_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl User {
    pub async fn count<'a, E>(
        filters: &QueryFilter<UserLookupFilter<'_>>,
        con: E,
    ) -> Result<i64, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let mut q = Query::select()
            .expr(Expr::asterisk().count())
            .from(Users)
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
        filters: &QueryFilter<UserLookupFilter<'_>>,
        con: E,
    ) -> Result<Vec<Self>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let (sql, args) = Query::select()
            .expr(Expr::asterisk())
            .from(Users)
            .to_owned()
            .apply_filters(filters)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub async fn get_by_id<'a, E>(id: Uuid, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(con)
            .await
    }

    pub async fn get_by_username<'a, E>(username: &str, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(con)
            .await
    }

    pub async fn create<'a, E>(user: impl Into<UserCreate>, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let user = user.into();
        let username = user.username.to_lowercase();
        let email = user.email.map(|val| val.to_lowercase());
        let hashed_password = helpers::hash(&user.password);

        sqlx::query_as!(
            Self,
            "INSERT INTO users (id, username, password, email) VALUES ($1, $2, $3, $4)
                RETURNING *",
            Uuid::new_v4(),
            username,
            hashed_password,
            email,
        )
        .fetch_one(con)
        .await
    }

    pub async fn update<'a, E>(id: Uuid, user: impl Into<UserUpdate>, con: E) -> Result<Self, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let user = user.into();
        let username = user.username.map(|val| val.to_lowercase());
        let hashed_password = user.password.map(|val| helpers::hash(&val));
        let email = user.email.map(|val| val.to_lowercase());

        sqlx::query_as!(
            Self,
            r#"
                UPDATE users
                SET username = COALESCE($1, username),
                    password = COALESCE($2, password),
                    email = COALESCE($3, email)
                WHERE id = $4
                RETURNING *
            "#,
            username,
            hashed_password,
            email,
            id
        )
        .fetch_one(con)
        .await
    }

    pub async fn delete<'a, E>(filters: &UserDeleteFilter, con: E) -> Result<Vec<User>, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        if filters.is_empty() {
            return Err(Error::RowNotFound);
        }

        let (sql, args) = Query::delete()
            .from_table(Users)
            .to_owned()
            .apply_conds(filters)
            .returning(Query::returning().expr(Expr::asterisk()))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with(&sql, args).fetch_all(con).await
    }

    pub async fn delete_by_id<'a, E>(id: Uuid, con: E) -> Result<User, Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        sqlx::query_as!(Self, "DELETE FROM users WHERE id = $1 RETURNING *", id)
            .fetch_one(con)
            .await
    }

    pub async fn update_login_timestamp<'a, E>(
        id: Uuid,
        con: E,
    ) -> Result<(Uuid, DateTime<Utc>), Error>
    where
        E: Executor<'a, Database = Postgres>,
    {
        let now = Utc::now();

        let row = sqlx::query!(
            r#"
                UPDATE users SET last_logged_in_at = $1
                WHERE id = $2
                RETURNING
                    id
            "#,
            now,
            id
        )
        .fetch_one(con)
        .await;

        Ok((row?.id, now))
    }

    pub fn check_password(&self, password: &str) -> bool {
        helpers::verify(password, &self.password)
    }
}
