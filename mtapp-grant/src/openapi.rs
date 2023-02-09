use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::OpenApi;

use crate::{admin, errors::GrantErrorOai, models::Grant, schemas::GrantList};

#[derive(OpenApi)]
#[openapi(
    info(description = "Grant management endpoints"),
    paths(admin::list, admin::create, admin::batch_delete, admin::delete),
    components(schemas(
        // Responses
        Grant,
        GrantList,

        // Params
        UuidFilterSet,
        DateTimeTzFilterSet,
        StringFilterSet,

        // Errors
        GrantErrorOai::NotFound,
        GrantErrorOai::AlreadyExist
    ))
)]
pub(crate) struct InternalGrantOpenApi;
