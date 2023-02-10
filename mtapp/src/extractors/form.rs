use std::ops::Deref;

use axum::{
    extract::{
        rejection::{BytesRejection, FailedToBufferBody, FormRejection},
        FromRequest,
    },
    http::Request,
    RequestExt,
};

use super::errors::{ErrorDetail, ExtractionError, Position};

/// A wrapper around axum's Form with custom rejection
#[derive(Debug, Clone, Copy, Default)]
pub struct Form<T>(pub T);

#[axum::async_trait]
impl<T, S, B> FromRequest<S, B> for Form<T>
where
    axum::Form<T>: FromRequest<S, B, Rejection = FormRejection>,
    S: Send + Sync,
    B: Send + 'static,
    T: 'static,
{
    type Rejection = ExtractionError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        match req.extract_with_state::<axum::Form<T>, S, _>(state).await {
            Ok(r) => return Ok(Form(r.0)),
            Err(FormRejection::InvalidFormContentType(_)) => {
                return Err(ExtractionError::InvalidContentType(ErrorDetail {
                    position: Position::General,
                    explain: "Expected `application/x-www-form-urlencoded`.".into(),
                }))
            }
            Err(FormRejection::FailedToDeserializeForm(_)) => {
                return Err(ExtractionError::FailedToDeserialize(ErrorDetail {
                    position: Position::General,
                    explain: "Failed to deserialize the form data.".into(),
                }))
            }
            Err(FormRejection::BytesRejection(e)) => match e {
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

            Err(_) => {}
        }
        Err(ExtractionError::UnknownError(ErrorDetail {
            position: Position::Body,
            explain: "Something went wrong while parsing the form data.".into(),
        }))
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
