use axum::{extract::Path, response::IntoResponse, Extension};
use json_response::{InternalErrorResponse, JsonListMeta, JsonResponse};
use mtapp_auth::{openapi_errors::AuthErrorAuthentication, Claims, TokenBlacklist};
use sqlx::{types::Uuid, PgPool};

use crate::{
    errors::{utoipa_response::SessionErrorNotFound, SessionError},
    models::Session,
};

#[utoipa::path(
    get,
    tag = "Session",
    path = "/",
    responses(
        (status = 200, body=SessionList),
        AuthErrorAuthentication,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user_id = claims.user_id;

    let sessions = Session::find_by_user(user_id, &pool).await?;
    let total = Session::count_by_user(user_id, &pool).await?;
    Ok(JsonResponse::with_content(sessions).meta(JsonListMeta::default().total(total as usize)))
}

#[utoipa::path(
    get,
    tag = "Session",
    path = "/{session_id}",
    params(
        ("session_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=Session),
        AuthErrorAuthentication,
        SessionErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get(
    session_id: Option<Path<Uuid>>,
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user_id = claims.user_id;

    let session = if let Some(sid) = session_id {
        Session::get_by_id_for_user(user_id, *sid, &pool).await?
    } else {
        let jti = claims.jti;
        Session::get_by_jti(jti, &pool).await?
    };

    Ok(JsonResponse::with_content(session))
}

#[utoipa::path(
    delete,
    tag = "Session",
    path = "/{session_id}",
    params(
        ("session_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=Session),
        AuthErrorAuthentication,
        SessionErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete(
    session_id: Path<Uuid>,
    claims: Claims,
    blacklist: TokenBlacklist,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    blacklist
        .blacklist(claims.jti)
        .await
        .map_err(|_| SessionError::InternalError)?;

    let user_id = claims.user_id;
    let deleted = Session::delete_by_id_for_user(user_id, *session_id, &pool).await?;

    Ok(JsonResponse::with_content(deleted))
}
