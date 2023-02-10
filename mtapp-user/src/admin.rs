use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Extension;
use basteh::Storage;
use json_resp::{CombineErrors, JsonListMeta, JsonResponse};
use seaqs::QueryFilter;
use sqlx::{types::Uuid, PgPool};
use validator::Validate;

use mtapp::extractors::{oai, Json, Query};
use mtapp_auth::AuthErrorOai;

use crate::errors::{UserError, UserErrorOai};
use crate::filters::{UserDeleteFilter, UserLookupFilter};
use crate::models::User;
use crate::schemas::{UserCreate, UserList, UserUpdate};

type QueryUserLookupFilter = QueryFilter<UserLookupFilter<'static>>;

#[utoipa::path(
    get,
    tag = "User",
    path = "/",
    params(
        QueryUserLookupFilter
    ),
    responses(
        (status = 200, body=inline(JsonResponse<UserList>)),
        oai::QueryErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        UserErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn list(
    Query(query): Query<QueryFilter<UserLookupFilter<'_>>>,
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::AllExtErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        CombineErrors::<UserErrorOai::DuplicateField, UserErrorOai::ValidationError>,
        UserErrorOai::InternalError
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
        (status = 200, body=inline(JsonResponse<UserList>)),
        oai::QueryErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        UserErrorOai::InternalError
    ),
    security(
        ("jwt_token" = [])
    )
)]
pub async fn batch_delete(
    Query(query): Query<UserDeleteFilter>,
    Extension(storage): Extension<Storage>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let users = User::delete(&query, &pool).await?;
    for user in users.iter() {
        storage.scope("banned_user_ids").set(user.id, 0).await?;
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::PathErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        UserErrorOai::NotFound,
        UserErrorOai::InternalError
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::AllExtErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        UserErrorOai::DuplicateField,
        UserErrorOai::ValidationError,
        UserErrorOai::NotFound,
        UserErrorOai::InternalError
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
        (status = 200, body=inline(JsonResponse<User>)),
        oai::PathErrors,
        AuthErrorOai::Authentication,
        AuthErrorOai::Permission,
        UserErrorOai::NotFound,
        UserErrorOai::InternalError
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
    storage.scope("banned_user_ids").set(user.id, 0).await?;

    Result::<_, UserError>::Ok(JsonResponse::with_content(user))
}
