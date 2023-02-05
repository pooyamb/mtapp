use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use basteh::Storage;
use json_response::{InternalErrorResponse, JsonListMeta, JsonResponse};
use mtapp_auth::openapi_errors::{AuthErrorAuthentication, AuthErrorPermission};
use seaqs::QueryFilter;
use serde_querystring_axum::QueryString;
use sqlx::{types::Uuid, PgPool};
use validator::Validate;

use crate::errors::{
    utoipa_response::{UserErrorDuplicateField, UserErrorNotFound, UserErrorValidationError},
    UserError,
};
use crate::filters::{UserDeleteFilter, UserLookupFilter};
use crate::models::User;
use crate::schemas::{UserCreate, UserUpdate};

type QueryUserLookupFilter = QueryFilter<UserLookupFilter<'static>>;

#[utoipa::path(
    get,
    tag = "User",
    path = "/",
    params(
        QueryUserLookupFilter
    ),
    responses(
        (status = 200, body=UserList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list(
    QueryString(query): QueryString<QueryFilter<UserLookupFilter<'_>>>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let users = User::find(&query, &pool).await?;
    let total = User::count(&query, &pool).await?;
    Result::<_, UserError>::Ok(
        JsonResponse::with_content(users).meta(JsonListMeta::default().total(total as usize)),
    )
}

#[utoipa::path(
    post,
    tag = "User",
    path = "/",
    request_body(
        content=inline(UserCreate),
        content_type="application/json",
        description="User create"
    ),
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        AuthErrorPermission,
        UserErrorDuplicateField,
        UserErrorValidationError,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn create(
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserCreate>,
) -> impl IntoResponse {
    user.validate()?;
    let user = User::create(user, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    delete,
    tag = "User",
    path = "/",
    params(
        UserDeleteFilter
    ),
    responses(
        (status = 200, body=UserList),
        AuthErrorAuthentication,
        AuthErrorPermission,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn batch_delete(
    QueryString(query): QueryString<UserDeleteFilter>,
    Extension(storage): Extension<Storage>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let users = User::delete(&query, &pool).await?;
    for user in users.iter() {
        storage.scope("banned_user_ids").set(user.id, "").await?;
    }
    Result::<_, UserError>::Ok(JsonResponse::with_content(users))
}

#[utoipa::path(
    get,
    tag = "User",
    path = "/{user_id}",
    params(
        ("user_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        AuthErrorPermission,
        UserErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn get(id: Path<Uuid>, Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let user = User::get_by_id(*id, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    post,
    tag = "User",
    path = "/{user_id}",
    params(
        ("user_id" = Uuid, Path,)
    ),
    request_body(
        content=inline(UserUpdate),
        content_type="application/json",
        description="User update"
    ),
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        AuthErrorPermission,
        UserErrorDuplicateField,
        UserErrorValidationError,
        UserErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn update(
    id: Path<Uuid>,
    Extension(pool): Extension<PgPool>,
    Json(user): Json<UserUpdate>,
) -> impl IntoResponse {
    user.validate()?;
    let user = User::update(*id, user, &pool).await?;
    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}

#[utoipa::path(
    delete,
    tag = "User",
    path = "/{user_id}",
    params(
        ("user_id" = Uuid, Path,)
    ),
    responses(
        (status = 200, body=User),
        AuthErrorAuthentication,
        AuthErrorPermission,
        UserErrorNotFound,
        InternalErrorResponse
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn delete(
    id: Path<Uuid>,
    Extension(storage): Extension<Storage>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let user = User::delete_by_id(*id, &pool).await?;
    storage.scope("banned_user_ids").set(user.id, "").await?;

    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}
