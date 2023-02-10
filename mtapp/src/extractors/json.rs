use std::ops::{Deref, DerefMut};

use axum::{
    extract::{
        rejection::{BytesRejection, FailedToBufferBody, JsonRejection},
        FromRequest,
    },
    http::Request,
    response::{IntoResponse, Response},
    RequestExt,
};
use serde::Serialize;

use super::errors::{ErrorDetail, ExtractionError, Position};

/// A wrapper around axum's Form with custom rejection
#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);

#[axum::async_trait]
impl<T, S, B> FromRequest<S, B> for Json<T>
where
    axum::Json<T>: FromRequest<S, B, Rejection = JsonRejection>,
    S: Send + Sync,
    B: Send + 'static,
    T: 'static,
{
    type Rejection = ExtractionError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        match req.extract_with_state::<axum::Json<T>, S, _>(state).await {
            Ok(r) => return Ok(Json(r.0)),
            Err(JsonRejection::JsonDataError(err)) => {
                return Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                    position: Position::Body,
                    explain: err.body_text().into(),
                }))
            }
            Err(JsonRejection::JsonSyntaxError(err)) => {
                return Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                    position: Position::Body,
                    explain: err.body_text().into(),
                }))
            }
            Err(JsonRejection::MissingJsonContentType(_)) => {
                return Err(ExtractionError::InvalidContentType(ErrorDetail {
                    position: Position::General,
                    explain: "Expected `application/json`.".into(),
                }))
            }
            Err(JsonRejection::BytesRejection(e)) => match e {
                BytesRejection::FailedToBufferBody(e) => match e {
                    FailedToBufferBody::LengthLimitError(_) => {
                        return Err(ExtractionError::LengthLimit)
                    }
                    FailedToBufferBody::UnknownBodyError(_) => {
                        return Err(ExtractionError::FailedToBuffer)
                    }
                    _ => {}
                },
                _ => {}
            },

            _ => {}
        }

        Err(ExtractionError::UnknownError(ErrorDetail {
            position: Position::Body,
            explain: "Something went wrong while parsing the json data.".into(),
        }))
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Json<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
