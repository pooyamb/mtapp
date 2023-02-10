use std::ops::Deref;

use axum::{
    extract::{rejection::PathRejection, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};

use super::errors::{ErrorDetail, ExtractionError, Position};

/// A wrapper around axum's Path with custom rejection
#[derive(Debug, Clone, Copy, Default)]
pub struct Path<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for Path<T>
where
    axum::extract::Path<T>: FromRequestParts<S, Rejection = PathRejection>,
    S: Send + Sync,
    T: 'static,
{
    type Rejection = ExtractionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match parts
            .extract_with_state::<axum::extract::Path<T>, S>(state)
            .await
        {
            Ok(r) => Ok(Path(r.0)),
            Err(PathRejection::MissingPathParams(e)) => {
                Err(ExtractionError::InternalError(Box::new(e)))
            }
            Err(PathRejection::FailedToDeserializePathParams(e)) => {
                Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                    position: Position::Path,
                    explain: e.body_text().into(),
                }))
            }
            Err(_) => Err(ExtractionError::UnknownError(ErrorDetail {
                position: Position::Path,
                explain: "Something went wrong while parsing the path parameters.".into(),
            })),
        }
    }
}

impl<T> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
