use mtapp::Uuid;
use serde::Deserialize;
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    ToSchema,
};

use crate::models::Grant;

#[derive(Deserialize, ToSchema)]
pub struct GrantCreate {
    pub(crate) user_id: Uuid,
    pub(crate) scope_id: Uuid,
}

#[derive(utoipa::ToResponse)]
pub(crate) struct GrantList(Vec<Grant>);

impl ToSchema<'static> for GrantList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "GrantList",
            ArrayBuilder::new().items(Grant::schema().1).build().into(),
        )
    }
}
