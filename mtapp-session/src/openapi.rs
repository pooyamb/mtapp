use json_response::JsonResponse;
use seaqs::filters::UuidFilterSet;
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    OpenApi, ToSchema,
};

use crate::{admin, errors::utoipa_response::SessionErrorNotFound, handlers, models::Session};

#[derive(utoipa::ToResponse)]
struct SessionList(Vec<Session>);

impl ToSchema<'static> for SessionList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "SessionList",
            ArrayBuilder::new()
                .items(Session::schema().1)
                .build()
                .into(),
        )
    }
}

type SessionJson = JsonResponse<Session>;
type SessionListJson = JsonResponse<SessionList>;

#[derive(OpenApi)]
#[openapi(
    info(description = "Session management endpoints"),
    paths(handlers::list, handlers::get, handlers::delete),
    components(schemas(SessionJson, SessionListJson), responses(SessionErrorNotFound),)
)]
pub(crate) struct PublicSessionOpenApi;

#[derive(OpenApi)]
#[openapi(
    info(description = "Session management endpoints"),
    paths(admin::list, admin::batch_delete, admin::get, admin::delete),
    components(
        schemas(SessionJson, SessionListJson, UuidFilterSet),
        responses(SessionErrorNotFound),
    )
)]
pub(crate) struct InternalSessionOpenApi;
