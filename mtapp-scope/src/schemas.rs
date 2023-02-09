use serde::Deserialize;
use utoipa::{
    openapi::{ArrayBuilder, RefOr, Schema},
    ToSchema,
};

use crate::Scope;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScopeCreate {
    pub name: String,
}

#[derive(utoipa::ToResponse)]
pub(crate) struct ScopeList(Vec<Scope>);

impl ToSchema<'static> for ScopeList {
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "ScopeList",
            ArrayBuilder::new().items(Scope::schema().1).build().into(),
        )
    }
}
