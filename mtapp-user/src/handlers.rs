use axum::{response::IntoResponse, Extension, Json};
use json_response::JsonResponse;
use mtapp_auth::Claims;
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::UserError,
    models::User,
    schemas::{SelfUpdate, UserRegister},
};

pub async fn signup(
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserRegister>,
) -> Result<impl IntoResponse, UserError> {
    user.validate()?;
    let user = User::create(user, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn get(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, UserError> {
    let user = User::get_by_id(claims.inner().user_id, &pool).await?;
    Ok(JsonResponse::with_content(user))
}

pub async fn update(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Json(user): Json<SelfUpdate>,
) -> Result<impl IntoResponse, UserError> {
    user.validate()?;
    let user = User::update(claims.inner().user_id, user, &pool).await?;
    Ok(JsonResponse::with_content(user))
}
