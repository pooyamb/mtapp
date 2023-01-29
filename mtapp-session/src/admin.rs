use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use json_response::{JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::errors::SessionError;
use crate::filters::{SessionDeleteFilter, SessionLookupFilter};
use crate::models::Session;

pub async fn list(
    QueryString(query): QueryString<QueryFilter<SessionLookupFilter>>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let users = Session::find(&query, &pool).await?;
    let total = Session::count(&query, &pool).await?;
    Ok(JsonResponse::with_content(users).meta(JsonListMeta::default().total(total as usize)))
}

pub async fn batch_delete(
    QueryString(query): QueryString<SessionDeleteFilter>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let sessions = Session::delete(&query, &pool).await?;

    Ok(JsonResponse::with_content(sessions))
}

pub async fn get(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user = Session::get_by_id(*id, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn delete(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let id = Session::delete_by_id(*id, &pool).await?;

    Ok(JsonResponse::with_content(id))
}
