use sea_query::Cond;
use seaqs::{
    filters::{DateTimeFilterSet, StringFilterSet, UuidFilterSet},
    Filter, ToCond, ToFieldCond,
};
use serde::Deserialize;

use crate::models::UserIden;

#[derive(Debug, Default, Deserialize)]
pub struct UserLookupFilter<'a> {
    username: Option<StringFilterSet<'a>>,
    email: Option<StringFilterSet<'a>>,
    last_logged_in_at: Option<DateTimeFilterSet>,
    created_at: Option<DateTimeFilterSet>,
    updated_at: Option<DateTimeFilterSet>,
}

impl ToCond for UserLookupFilter<'_> {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(username) = self.username.to_cond(UserIden::Username) {
            cond = cond.add(username)
        }
        if let Some(email) = self.email.to_cond(UserIden::Email) {
            cond = cond.add(email)
        }
        if let Some(last_logged_in_at) = self.last_logged_in_at.to_cond(UserIden::LastLoggedInAt) {
            cond = cond.add(last_logged_in_at)
        }
        if let Some(created_at) = self.created_at.to_cond(UserIden::CreatedAt) {
            cond = cond.add(created_at)
        }
        if let Some(updated_at) = self.updated_at.to_cond(UserIden::UpdatedAt) {
            cond = cond.add(updated_at)
        }
        cond
    }
}

impl Filter for UserLookupFilter<'_> {
    const SORTABLE_FIELDS: &'static [&'static str] = &[
        "username",
        "email",
        "last_logged_in_at",
        "created_at",
        "updated_at",
    ];
}

#[derive(Debug, Deserialize)]
pub struct UserDeleteFilter {
    id: UuidFilterSet,
}

impl ToCond for UserDeleteFilter {
    fn to_cond(&self) -> Cond {
        let mut cond = Cond::all();
        if let Some(ids) = self.id.to_cond(UserIden::Id) {
            cond = cond.add(ids)
        }
        cond
    }
}

impl UserDeleteFilter {
    pub fn is_empty(&self) -> bool {
        self.id.is_empty()
    }
}
