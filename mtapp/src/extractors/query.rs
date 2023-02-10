use std::ops::Deref;

use axum::{extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;
use serde_querystring::BracketsQS;

use super::errors::{ErrorDetail, ExtractionError, Position};

#[derive(Debug, Clone, Copy, Default)]
pub struct Query<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ExtractionError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        let value = BracketsQS::parse(query.as_bytes())
            .deserialize()
            .map_err(|e| {
                ExtractionError::FailedToDeserialize(ErrorDetail {
                    position: Position::Query,
                    explain: e.to_string().into(),
                })
            })?;
        Ok(Query(value))
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
