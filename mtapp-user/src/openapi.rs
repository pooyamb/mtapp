use json_response::JsonResponse;
use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    OpenApi, ToSchema,
};

use crate::{
    admin,
    errors::utoipa_response::{
        UserErrorDuplicateField, UserErrorNotFound, UserErrorValidationError,
    },
    handlers,
    models::User,
};

#[derive(utoipa::ToResponse)]
struct UserList(Vec<User>);

impl ToSchema<'static> for UserList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "UserList",
            ArrayBuilder::new().items(User::schema().1).build().into(),
        )
    }
}

type UserJson = JsonResponse<User>;
type UserListJson = JsonResponse<UserList>;

#[derive(OpenApi)]
#[openapi(
    info(description = "User management endpoints"),
    paths(handlers::signup, handlers::get, handlers::update),
    components(
        schemas(UserJson),
        responses(UserErrorNotFound, UserErrorValidationError, UserErrorDuplicateField)
    )
)]
pub(crate) struct PublicUserOpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "User management endpoints"),
    paths(
        admin::list,
        admin::create,
        admin::batch_delete,
        admin::get,
        admin::update,
        admin::delete
    ),
    components(
        schemas(
            UserJson,
            UserListJson,
            UuidFilterSet,
            DateTimeTzFilterSet,
            StringFilterSet
        ),
        responses(UserErrorNotFound, UserErrorValidationError, UserErrorDuplicateField),
    )
)]
pub(crate) struct InternalUserOpenApi;
