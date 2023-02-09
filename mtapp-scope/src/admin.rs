use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use json_resp::{JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::types::Uuid;
use sqlx::PgPool;

use mtapp_auth::AuthErrorOai;

use crate::errors::{ScopeError, ScopeErrorOai};
use crate::filters::{ScopeDeleteFilter, ScopeLookupFilter};
use crate::models::Scope;
use crate::schemas::{ScopeCreate, ScopeList};

type QueryScopeLookupFilter = QueryFilter<ScopeLookupFilter<'static>>;

#[utoipa::path(
    get,
    tag = "Scope",
    path = "/",
    params(
        QueryScopeLookupFilter
    ),
    responses(
        (status = 200, body=inline(JsonResponse<ScopeList>)),
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        ScopeErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<QueryFilter<ScopeLookupFilter<'_>>>,
) -> impl IntoResponse {
    let scopes = Scope::find(&query, &pool).await?;
    let total = Scope::count(&query, &pool).await?;
    Result::<_, ScopeError>::Ok(
        JsonResponse::with_content(scopes).meta(JsonListMeta::default().total(total as usize)),
    )
}

#[utoipa::path(
    post,
    tag = "Scope",
    path = "/",
    request_body(
        content=inline(ScopeCreate),
        content_type="application/json",
        description="Scope create"
    ),
    responses(
        (status = 200, body=inline(JsonResponse<Scope>)),
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        ScopeErrorOai::DuplicateField,
        ScopeErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(scope): Json<ScopeCreate>,
) -> impl IntoResponse {
    let scope = Scope::create(scope.name, &pool).await?;
    Result::<_, ScopeError>::Ok(JsonResponse::with_content(scope))
}

#[utoipa::path(
    delete,
    tag = "Scope",
    path = "/",
    params(
        ScopeDeleteFilter
    ),
    responses(
        (status = 200, body=inline(JsonResponse<ScopeList>)),
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        ScopeErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn batch_delete(
    Extension(pool): Extension<PgPool>,
    QueryString(query): QueryString<ScopeDeleteFilter>,
) -> impl IntoResponse {
    let scopes = Scope::delete(&query, &pool).await?;
    Result::<_, ScopeError>::Ok(JsonResponse::with_content(scopes))
}

#[utoipa::path(
    get,
    tag = "Scope",
    path = "/{scope_id}",
    params(
        ("scope_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=inline(JsonResponse<Scope>)),
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        ScopeErrorOai::NotFound,
        ScopeErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get(id: Path<Uuid>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let scope = Scope::get_by_id(*id, &pool).await?;
    Result::<_, ScopeError>::Ok(JsonResponse::with_content(scope))
}

#[utoipa::path(
    delete,
    tag = "Scope",
    path = "/{scope_id}",
    params(
        ("scope_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=inline(JsonResponse<Scope>)),
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        ScopeErrorOai::NotFound,
        ScopeErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete(id: Path<Uuid>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let scope = Scope::delete_by_id(*id, &pool).await?;
    Result::<_, ScopeError>::Ok(JsonResponse::with_content(scope))
}
