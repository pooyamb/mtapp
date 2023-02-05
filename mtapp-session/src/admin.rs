use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use json_response::{InternalErrorResponse, JsonListMeta, JsonResponse};
use mtapp_auth::{
    openapi_errors::{AuthErrorAuthentication, AuthErrorPermission},
    TokenBlacklist,
};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::errors::{utoipa_response::SessionErrorNotFound, SessionError};
use crate::filters::{SessionDeleteFilter, SessionLookupFilter};
use crate::models::Session;

type QuerySessionLookupFilter = QueryFilter<SessionLookupFilter>;

#[utoipa::path(
    get,
    tag = "Session",
    path = "/",
    params(
        QuerySessionLookupFilter
    ),
    responses(
        (status = 200, body=SessionList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list(
    QueryString(query): QueryString<QueryFilter<SessionLookupFilter>>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let users = Session::find(&query, &pool).await?;
    let total = Session::count(&query, &pool).await?;
    Ok(JsonResponse::with_content(users).meta(JsonListMeta::default().total(total as usize)))
}

#[utoipa::path(
    delete,
    tag = "Session",
    path = "/",
    params(
        SessionDeleteFilter
    ),
    responses(
        (status = 200, body=SessionList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn batch_delete(
    QueryString(query): QueryString<SessionDeleteFilter>,
    Extension(pool): Extension<PgPool>,
    blacklist: TokenBlacklist,
) -> Result<impl IntoResponse, SessionError> {
    let sessions = Session::delete(&query, &pool).await?;

    for session in sessions.iter() {
        blacklist
            .blacklist(session.jti)
            .await
            .map_err(|_| SessionError::InternalError)?;
    }

    Ok(JsonResponse::with_content(sessions))
}

#[utoipa::path(
    get,
    tag = "Session",
    path = "/{session_id}",
    params(
        ("session_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=SessionList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        SessionErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user = Session::get_by_id(*id, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    delete,
    tag = "Session",
    path = "/{session_id}",
    params(
        ("session_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=SessionList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        SessionErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete(
    id: Path<Uuid>,
    blacklist: TokenBlacklist,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let session = Session::delete_by_id(*id, &pool).await?;

    blacklist
        .blacklist(session.jti)
        .await
        .map_err(|_| SessionError::InternalError)?;

    Ok(JsonResponse::with_content(session))
}
