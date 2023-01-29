use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use json_response::{JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::errors::ScopeError;
use crate::filters::{ScopeDeleteFilter, ScopeLookupFilter};
use crate::models::Scope;
use crate::schemas::ScopeCreate;

pub async fn list(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<QueryFilter<ScopeLookupFilter<'_>>>,
) -> Result<impl IntoResponse, ScopeError> {
    let users = Scope::find(&query, &pool).await?;
    let total = Scope::count(&query, &pool).await?;
    Ok(JsonResponse::with_content(users).meta(JsonListMeta::default().total(total as usize)))
}

pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(scope): Json<ScopeCreate>,
) -> Result<impl IntoResponse, ScopeError> {
    let scope = Scope::create(scope.name, &pool).await?;
    Ok(JsonResponse::with_content(scope))
}

pub async fn batch_delete(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<ScopeDeleteFilter>,
) -> Result<impl IntoResponse, ScopeError> {
    let scopes = Scope::delete(&query, &pool).await?;

    Ok(JsonResponse::with_content(scopes))
}

pub async fn get(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, ScopeError> {
    let user = Scope::get_by_id(*id, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn delete(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, ScopeError> {
    let scope = Scope::delete_by_id(*id, &pool).await?;

    Ok(JsonResponse::with_content(scope))
}
