use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    ToSchema,
};

use crate::models::Session;

#[derive(utoipa::ToResponse)]
pub(crate) struct SessionList(Vec<Session>);

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
