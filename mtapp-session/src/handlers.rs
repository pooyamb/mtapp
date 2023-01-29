use axum::{extract::Path, response::IntoResponse, Extension};
use json_response::{JsonListMeta, JsonResponse};
use mtapp_auth::{Claims, ClaimsModify};
use sqlx::{types::Uuid, PgPool};

use crate::{errors::SessionError, models::Session};

pub async fn list(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user_id = claims.inner().user_id;

    let sessions = Session::find_by_user(user_id, &pool).await?;
    let total = Session::count_by_user(user_id, &pool).await?;
    Ok(JsonResponse::with_content(sessions).meta(JsonListMeta::default().total(total as usize)))
}

pub async fn get(
    session_id: Option<Path<Uuid>>,
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    let user_id = claims.inner().user_id;

    let session = if let Some(sid) = session_id {
        Session::get_by_id_for_user(user_id, *sid, &pool).await?
    } else {
        let jti = claims.inner().jti;
        Session::get_by_jti(jti, &pool).await?
    };

    Ok(JsonResponse::with_content(session))
}

pub async fn delete(
    session_id: Path<Uuid>,
    claims: ClaimsModify,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, SessionError> {
    claims
        .invalidate()
        .await
        .map_err(|_| SessionError::InternalError)?;

    let user_id = claims.get_claims().inner().user_id;
    let deleted = Session::delete_by_id_for_user(user_id, *session_id, &pool).await?;

    Ok(JsonResponse::with_content(deleted))
}
