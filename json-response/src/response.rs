use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

fn as_u16<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u16(status.as_u16())
}

#[derive(Debug, Default, Serialize, ToSchema)]
pub struct Nothing;

#[derive(Debug, Default, Serialize)]
pub struct JsonListMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prev: Option<String>,
}

impl JsonListMeta {
    pub fn next(mut self, next: String) -> Self {
        self.next = Some(next);
        self
    }

    pub fn prev(mut self, prev: String) -> Self {
        self.prev = Some(prev);
        self
    }

    pub fn total(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }
}

#[derive(Debug, Serialize)]
pub struct JsonResponse<T, M = Nothing> {
    #[serde(serialize_with = "as_u16")]
    status: StatusCode,
    content: T,
    meta: M,
}

impl<T, M> Default for JsonResponse<T, M>
where
    T: Default,
    M: Default,
{
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            content: T::default(),
            meta: M::default(),
        }
    }
}

impl JsonResponse<()> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T> JsonResponse<T> {
    pub fn with_content(content: T) -> Self {
        Self {
            status: StatusCode::OK,
            content,
            meta: Nothing,
        }
    }
}

impl<T, M> JsonResponse<T, M> {
    pub fn content<T2>(self, content: T2) -> JsonResponse<T2, M> {
        JsonResponse {
            status: self.status,
            content,
            meta: self.meta,
        }
    }

    pub fn meta<M2>(self, meta: M2) -> JsonResponse<T, M2> {
        JsonResponse {
            status: self.status,
            content: self.content,
            meta,
        }
    }
}

impl<T, M> fmt::Display for JsonResponse<T, M>
where
    T: Serialize,
    M: Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JsonResponse: ")?;
        f.write_str(&serde_json::to_string_pretty(self).map_err(|_err| fmt::Error)?)
    }
}

impl<T, M> IntoResponse for JsonResponse<T, M>
where
    T: Serialize,
    M: Serialize,
{
    fn into_response(self) -> Response {
        (self.status, Json(&self)).into_response()
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, Serialize)]
pub struct JsonError<T = Nothing> {
    #[serde(serialize_with = "as_u16")]
    pub status: StatusCode,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub content: T,
}

impl JsonError {
    pub fn new(status: StatusCode, code: &'static str) -> Self {
        Self {
            status,
            code,
            ..Default::default()
        }
    }
}

impl<T> JsonError<T> {
    pub fn with_content(status: StatusCode, code: &'static str, content: T) -> Self {
        Self {
            status,
            code,
            hint: None,
            content,
        }
    }

    pub fn hint(mut self, hint: String) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn content<B>(self, content: B) -> JsonError<B> {
        JsonError {
            status: self.status,
            code: self.code,
            hint: self.hint,
            content,
        }
    }
}

impl<T> fmt::Display for JsonError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JsonError: ")?;
        f.write_str(self.code)
    }
}

impl<T> IntoResponse for JsonError<T>
where
    T: Serialize + 'static,
{
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(&self)).into_response()
    }
}
