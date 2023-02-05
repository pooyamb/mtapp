use json_response::JsonResponse;
use seaqs::filters::{DateTimeTzFilterSet, StringFilterSet, UuidFilterSet};
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    OpenApi, ToSchema,
};

use crate::{
    admin,
    errors::utoipa_response::{GrantErrorAlreadyExist, GrantErrorNotFound},
    models::Grant,
};

#[derive(utoipa::ToResponse)]
struct GrantList(Vec<Grant>);

impl ToSchema<'static> for GrantList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "GrantList",
            ArrayBuilder::new().items(Grant::schema().1).build().into(),
        )
    }
}

type GrantJson = JsonResponse<Grant>;
type GrantListJson = JsonResponse<GrantList>;

#[derive(OpenApi)]
#[openapi(
    info(description = "Grant management endpoints"),
    paths(admin::list, admin::create, admin::batch_delete, admin::delete),
    components(
        schemas(
            GrantJson,
            GrantListJson,
            UuidFilterSet,
            DateTimeTzFilterSet,
            StringFilterSet
        ),
        responses(GrantErrorNotFound, GrantErrorAlreadyExist),
    )
)]
pub(crate) struct InternalGrantOpenApi;
