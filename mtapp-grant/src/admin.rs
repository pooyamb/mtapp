use axum::{extract::Path, response::IntoResponse, Extension, Json};
use json_response::{InternalErrorResponse, JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::PgPool;

use mtapp::Uuid;
use mtapp_auth::openapi_errors::{AuthErrorAuthentication, AuthErrorPermission};

use crate::{
    errors::{
        utoipa_response::{GrantErrorAlreadyExist, GrantErrorNotFound},
        GrantError,
    },
    filters::{GrantDeleteFilter, GrantLookupFilter},
    models::Grant,
    schemas::GrantCreate,
};

type QueryGrantLookupFilter = QueryFilter<GrantLookupFilter>;

#[utoipa::path(
    get,
    tag = "Grant",
    path = "/",
    params(
        QueryGrantLookupFilter
    ),
    responses(
        (status = 200, body=GrantList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
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

#[utoipa::path(
    post,
    tag = "Grant",
    path = "/",
    request_body(
        content=inline(GrantCreate),
        content_type="application/json",
        description="Grant create"
    ),
    responses(
        (status = 200, body=Grant),
        AuthErrorAuthentication,
        AuthErrorPermission,
        GrantErrorAlreadyExist,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(scope): Json<GrantCreate>,
) -> impl IntoResponse {
    let grant = Grant::create(scope, &pool).await?;
    Result::<_, GrantError>::Ok(JsonResponse::with_content(grant))
}

#[utoipa::path(
    delete,
    tag = "Grant",
    path = "/",
    params(
        GrantDeleteFilter
    ),
    responses(
        (status = 200, body=GrantList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn batch_delete(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<GrantDeleteFilter>,
) -> impl IntoResponse {
    let scopes = Grant::delete(&query, &pool).await?;
    Result::<_, GrantError>::Ok(JsonResponse::with_content(scopes))
}

#[utoipa::path(
    delete,
    tag = "Grant",
    path = "/{grant_id}",
    params(
        ("grant_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=Grant),
        AuthErrorAuthentication,
        AuthErrorPermission,
        GrantErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete(
    Path(grant_id): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let grant = Grant::delete_by_id(grant_id, &pool)
        .await
        .map(JsonResponse::with_content)?;

    Result::<_, GrantError>::Ok(JsonResponse::with_content(grant))
}
