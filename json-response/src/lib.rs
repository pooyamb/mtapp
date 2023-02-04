//! This crate contains a set of structs and macros to ease the implementation of REST apis

mod response;
mod utoipa_impls;

pub use json_response_derive::ApiError;
pub use response::{JsonError, JsonListMeta, JsonResponse, Nothing};
pub use utoipa_impls::InternalErrorResponse;

/// Re-export utoipa
pub use utoipa;

pub type JsonResult<T, E = Nothing> = Result<JsonResponse<T>, JsonError<E>>;

#[doc(hidden)]
pub mod __private {
    pub use axum::http::StatusCode;
    pub use axum::response::{IntoResponse, Response};
    pub use log::error;

    pub mod utoipa {
        pub use utoipa::{
            openapi::{
                ContentBuilder, KnownFormat, ObjectBuilder, RefOr, Response, ResponseBuilder,
                Schema, SchemaFormat, SchemaType,
            },
            ToResponse, ToSchema,
        };
    }
}
