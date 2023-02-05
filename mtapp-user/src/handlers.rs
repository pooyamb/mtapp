use axum::{response::IntoResponse, Extension, Json};
use json_response::{InternalErrorResponse, JsonResponse};
use mtapp_auth::{openapi_errors::AuthErrorAuthentication, Claims};
use sqlx::PgPool;
use validator::Validate;

use crate::{
    errors::{
        utoipa_response::{UserErrorDuplicateField, UserErrorValidationError},
        UserError,
    },
    models::User,
    schemas::{SelfUpdate, UserRegister},
};

#[utoipa::path(
    post,
    tag = "User",
    path = "/",
    request_body(
        content=inline(UserRegister),
        content_type="application/json",
        description="User register"
    ),
    responses(
        (status = 200, body=User),
        UserErrorDuplicateField,
        UserErrorValidationError,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn signup(
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserRegister>,
) -> impl IntoResponse {
    user.validate()?;
    let user = User::create(user, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    get,
    tag = "User",
    path = "/me",
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get(claims: Claims, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let user = User::get_by_id(claims.user_id, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    post,
    tag = "User",
    path = "/me",
    request_body(
        content=inline(SelfUpdate),
        content_type="application/json",
        description="User register"
    ),
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update(
    claims: Claims,
    Extension(pool): Extension<PgPool>,
    Json(user): Json<SelfUpdate>,
) -> impl IntoResponse {
    user.validate()?;
    let user = User::update(claims.user_id, user, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}
