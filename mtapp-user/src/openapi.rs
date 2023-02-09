use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::OpenApi;

use crate::{admin, errors::UserErrorOai, handlers, models::User, schemas::UserList};

#[derive(OpenApi)]
#[openapi(
    info(description = "User management endpoints"),
    paths(handlers::signup, handlers::get, handlers::update),
    components(schemas(
        // Response
        User,

        // Errors
        UserErrorOai::NotFound,
        UserErrorOai::ValidationError,
        UserErrorOai::DuplicateField
    ))
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
    components(schemas(
        // Response
        User,
        UserList,

        // Params
        UuidFilterSet,
        DateTimeTzFilterSet,
        StringFilterSet,

        // Errors
        UserErrorOai::NotFound,
        UserErrorOai::ValidationError,
        UserErrorOai::DuplicateField
    ))
)]
pub(crate) struct InternalUserOpenApi;
