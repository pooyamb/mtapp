use axum::{extract::Path, response::IntoResponse, Extension, Json};
use json_response::JsonResponse;
use sqlx::PgPool;

use mtapp::Uuid;

use crate::{errors::GrantError, models::Grant, schemas::GrantCreate};

pub async fn list(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, GrantError> {
    Ok(Grant::get_grants(*id, &pool)
        .await
        .map(JsonResponse::with_content)?)
}

pub async fn create(
    user_id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(scope): Json<GrantCreate>,
) -> Result<impl IntoResponse, GrantError> {
    let grant = Grant::add_grant(*user_id, &scope.scope_name, &pool).await?;
    Ok(JsonResponse::with_content(grant))
}

pub async fn delete(
    Path((user_id, scope)): Path<(Uuid, String)>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, GrantError> {
    Grant::del_grant(user_id, &scope, &pool)
        .await
        .map(JsonResponse::with_content)?;

    let message = format!(
        "Grant of user with id: {} from scope with name:{} has been deleted",
        user_id, scope
    );

    Ok(JsonResponse::with_content(message))
}
