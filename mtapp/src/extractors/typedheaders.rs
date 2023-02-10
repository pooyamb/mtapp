use std::ops::Deref;

use axum::{
    extract::{
        rejection::{TypedHeaderRejection, TypedHeaderRejectionReason},
        FromRequestParts,
    },
    http::request::Parts,
    RequestPartsExt,
};

use super::errors::{ErrorDetail, ExtractionError, Position};

/// A wrapper around axum's TypedHeader with custom rejection
#[derive(Debug, Clone, Copy, Default)]
pub struct TypedHeader<T>(pub T);

#[axum::async_trait]
impl<T, S> FromRequestParts<S> for TypedHeader<T>
where
    axum::extract::TypedHeader<T>: FromRequestParts<S, Rejection = TypedHeaderRejection>,
    S: Send + Sync,
    T: 'static,
{
    type Rejection = ExtractionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match parts
            .extract_with_state::<axum::extract::TypedHeader<T>, S>(state)
            .await
        {
            Ok(r) => Ok(TypedHeader(r.0)),
            Err(err) => {
                let name = err.name();
                match err.reason() {
                    TypedHeaderRejectionReason::Missing => {
                        Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                            position: Position::Header,
                            explain: format!("Required header, {} is missing.", name).into(),
                        }))
                    }
                    TypedHeaderRejectionReason::Error(_) => {
                        Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                            position: Position::Header,
                            explain: format!("Failed to deserialize the header {}.", name).into(),
                        }))
                    }
                    _ => Err(ExtractionError::UnknownError(ErrorDetail {
                        position: Position::Header,
                        explain: "Something went wrong while parsing the headers.".into(),
                    })),
                }
            }
        }
    }
}

impl<T> Deref for TypedHeader<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
