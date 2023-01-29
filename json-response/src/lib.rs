//! This crate contains a set of structs and macros to ease the implementation of REST apis

mod response;

pub use json_response_derive::ApiError;
pub use response::{JsonError, JsonListMeta, JsonResponse};
pub type JsonResult<T, E = ()> = Result<JsonResponse<T>, JsonError<E>>;

#[doc(hidden)]
pub mod __private {
    pub use axum::http::StatusCode;
    pub use axum::response::{IntoResponse, Response};
}
