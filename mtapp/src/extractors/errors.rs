use std::{borrow::Cow, error::Error};

use axum::http::StatusCode;
use json_resp::JsonError;
use serde::Serialize;

fn is_general(p: &Position) -> bool {
    *p == Position::General
}

#[derive(Default, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    #[default]
    General,
    Path,
    Header,
    Query,
    Body,
}

#[derive(Default, Debug, Serialize)]
pub struct ErrorDetail {
    #[serde(skip_serializing_if = "is_general")]
    pub position: Position,
    pub explain: Cow<'static, str>,
}

#[derive(Debug, JsonError)]
#[json_error(internal_code = "500000 internal-error")]
pub enum ExtractionError {
    #[json_error(request, status = 422, code = "422000 deserialize-failed")]
    FailedToDeserialize(ErrorDetail),

    #[json_error(request, status = 415, code = "415000 invalid-content-type")]
    InvalidContentType(ErrorDetail),

    #[json_error(request, status = 413, code = "413000 length-limit-reached")]
    LengthLimit,

    #[json_error(request, status = 400, code = "400001 bufferring-failed")]
    FailedToBuffer,

    #[json_error(request, status = 400, code = "400000 decoding-error")]
    UnknownError(ErrorDetail),

    #[json_error(internal)]
    InternalError(Box<dyn Error + Send + Sync + 'static>),
}
