use sea_query::Cond;
use seaqs::{filters::UuidFilterSet, Filter, ToCond, ToFieldCond};
use serde::Deserialize;

use crate::models::SessionIden;

#[derive(Debug, Default, Deserialize)]
pub struct SessionLookupFilter {
    user_id: Option<UuidFilterSet>,
}

impl ToCond for SessionLookupFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(user_id) = self.user_id.to_cond(SessionIden::UserId) {
            cond = cond.add(user_id)
        }
        cond
    }
}

impl Filter for SessionLookupFilter {
    const SORTABLE_FIELDS: &'static [&'static str] =
        &["user_id", "ip", "last_access_at", "updated_at"];
}

#[derive(Debug, Deserialize)]
pub struct SessionDeleteFilter {
    id: UuidFilterSet,
}

impl ToCond for SessionDeleteFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(ids) = self.id.to_cond(SessionIden::Id) {
            cond = cond.add(ids)
        }
        cond
    }
}

impl SessionDeleteFilter {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
}
