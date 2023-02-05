use sea_query::Cond;
use seaqs::{
    filters::{DateTimeFilterSet, StringFilterSet, UuidFilterSet},
    Filter, ToCond, ToFieldCond,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::models::ScopeIden;

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct ScopeLookupFilter<'a> {
    name: Option<StringFilterSet<'a>>,
    created_at: Option<DateTimeFilterSet>,
}

impl<'a> ToCond for ScopeLookupFilter<'a> {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(name) = self.name.to_cond(ScopeIden::Name) {
            cond = cond.add(name);
        }
        if let Some(created_at) = self.created_at.to_cond(ScopeIden::CreatedAt) {
            cond = cond.add(created_at);
        }
        cond
    }
}

impl<'a> Filter for ScopeLookupFilter<'a> {
    const SORTABLE_FIELDS: &'static [&'static str] = &["name", "created_at"];
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ScopeDeleteFilter {
    #[param(style = DeepObject, inline, explode)]
    id: UuidFilterSet,
}

impl ToCond for ScopeDeleteFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(ids) = self.id.to_cond(ScopeIden::Id) {
            cond = cond.add(ids);
        }
        cond
    }
}

impl ScopeDeleteFilter {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
}
