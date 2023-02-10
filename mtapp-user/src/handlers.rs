use axum::{response::IntoResponse, Extension};
use json_resp::{CombineErrors, JsonResponse};
use sqlx::PgPool;
use validator::Validate;

use mtapp::extractors::{oai, Json};
use mtapp_auth::{AuthErrorOai, Claims};

use crate::{
    errors::{UserError, UserErrorOai},
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::AllExtErrors,
        CombineErrors::<UserErrorOai::DuplicateField, UserErrorOai::ValidationError>,
        UserErrorOai::InternalError
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
        (status = 200, body=inline(JsonResponse<User>)),
        AuthErrorOai::Authentication,
        UserErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get_me(claims: Claims, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::AllExtErrors,
        AuthErrorOai::Authentication,
        UserErrorOai::ValidationError,
        UserErrorOai::InternalError
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
