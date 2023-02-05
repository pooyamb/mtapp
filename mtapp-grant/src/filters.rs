use sea_query::Cond;
use seaqs::{filters::UuidFilterSet, Filter, ToCond, ToFieldCond};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::models::GrantIden;

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct GrantLookupFilter {
    user_id: Option<UuidFilterSet>,
    scope_id: Option<UuidFilterSet>,
}

impl<'a> ToCond for GrantLookupFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(user_id) = self.user_id.to_cond(GrantIden::UserId) {
            cond = cond.add(user_id);
        }
        if let Some(scope_id) = self.scope_id.to_cond(GrantIden::ScopeId) {
            cond = cond.add(scope_id);
        }
        cond
    }
}

impl<'a> Filter for GrantLookupFilter {
    const SORTABLE_FIELDS: &'static [&'static str] = &["user_id", "scope_id"];
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GrantDeleteFilter {
    #[param(style = DeepObject, inline, explode)]
    id: UuidFilterSet,
}

impl ToCond for GrantDeleteFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(ids) = self.id.to_cond(GrantIden::Id) {
            cond = cond.add(ids);
        }
        cond
    }
}

impl GrantDeleteFilter {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
}
