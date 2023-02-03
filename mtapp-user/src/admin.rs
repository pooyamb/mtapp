use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use basteh::Storage;
use json_response::{JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::{types::Uuid, PgPool};
use validator::Validate;

use crate::errors::UserError;
use crate::filters::{UserDeleteFilter, UserLookupFilter};
use crate::models::User;
use crate::schemas::{UserCreate, UserUpdate};

pub async fn list(
    QueryString(query): QueryString<QueryFilter<UserLookupFilter<'_>>>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let users = User::find(&query, &pool).await?;
    let total = User::count(&query, &pool).await?;
    Ok(JsonResponse::with_content(users).meta(JsonListMeta::default().total(total as usize)))
}

pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserCreate>,
) -> Result<impl IntoResponse, UserError> {
    user.validate()?;
    let user = User::create(user, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn batch_delete(
    QueryString(query): QueryString<UserDeleteFilter>,
    Extension(storage): Extension<Storage>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let users = User::delete(&query, &pool).await?;
    for user in users.iter() {
        storage.scope("banned_user_ids").set(user.id, "").await?;
    }
    Ok(JsonResponse::with_content(users))
}

pub async fn get(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let user = User::get_by_id(*id, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn update(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserUpdate>,
) -> Result<impl IntoResponse, UserError> {
    user.validate()?;
    let user = User::update(*id, user, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn delete(
    id: Path<Uuid>,
    Extension(storage): Extension<Storage>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let user = User::delete_by_id(*id, &pool).await?;
    storage.scope("banned_user_ids").set(user.id, "").await?;

    Ok(JsonResponse::with_content(user))
}
