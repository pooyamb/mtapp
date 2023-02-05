use json_response::JsonResponse;
use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    OpenApi, ToSchema,
};

use crate::{
    admin,
    errors::utoipa_response::{ScopeErrorDuplicateField, ScopeErrorNotFound},
    models::Scope,
};

#[derive(utoipa::ToResponse)]
struct ScopeList(Vec<Scope>);

impl ToSchema<'static> for ScopeList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "ScopeList",
            ArrayBuilder::new().items(Scope::schema().1).build().into(),
        )
    }
}

type ScopeJson = JsonResponse<Scope>;
type ScopeListJson = JsonResponse<ScopeList>;

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
    components(
        schemas(
            ScopeJson,
            ScopeListJson,
            UuidFilterSet,
            DateTimeTzFilterSet,
            StringFilterSet
        ),
        responses(ScopeErrorNotFound, ScopeErrorDuplicateField),
    )
)]
pub(crate) struct InternalScopeOpenApi;
