use axum::{extract::Path, response::IntoResponse, Extension, Json};
use json_response::{JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::PgPool;

use mtapp::Uuid;

use crate::{
    errors::GrantError,
    filters::{GrantDeleteFilter, GrantLookupFilter},
    models::Grant,
    schemas::GrantCreate,
};

pub async fn list(
    QueryString(query): QueryString<QueryFilter<GrantLookupFilter>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let grants = Grant::find(&query, &pool).await?;
    let count = Grant::count(&query, &pool).await?;

    Result::<_, GrantError>::Ok(
        JsonResponse::with_content(grants).meta(JsonListMeta::default().total(count as usize)),
    )
}

pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(scope): Json<GrantCreate>,
) -> impl IntoResponse {
    let grant = Grant::create(scope, &pool).await?;
    Result::<_, GrantError>::Ok(JsonResponse::with_content(grant))
}

pub async fn batch_delete(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<GrantDeleteFilter>,
) -> impl IntoResponse {
    let scopes = Grant::delete(&query, &pool).await?;
    Result::<_, GrantError>::Ok(JsonResponse::with_content(scopes))
}

pub async fn delete(
    Path(grant_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let grant = Grant::delete_by_id(grant_id, &pool)
        .await
        .map(JsonResponse::with_content)?;

    Result::<_, GrantError>::Ok(JsonResponse::with_content(grant))
}
