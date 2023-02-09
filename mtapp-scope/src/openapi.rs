use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::OpenApi;

use crate::{admin, errors::ScopeErrorOai, models::Scope, schemas::ScopeList};

#[derive(OpenApi)]
#[openapi(
    info(description = "Scope management endpoints"),
    paths(
        admin::list,
        admin::create,
        admin::batch_delete,
        admin::get,
        admin::delete
    ),
    components(schemas(
        // Responses
        Scope,
        ScopeList,

        // Params
        UuidFilterSet,
        DateTimeTzFilterSet,
        StringFilterSet,

        // Errors
        ScopeErrorOai::NotFound,
        ScopeErrorOai::DuplicateField
    ),)
)]
pub(crate) struct InternalScopeOpenApi;
