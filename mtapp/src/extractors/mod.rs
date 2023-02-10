mod errors;
mod form;
mod json;
mod path;
mod query;
mod typedheaders;

pub use form::Form;
pub use json::Json;
pub use path::Path;
pub use query::Query;
pub use typedheaders::TypedHeader;

mod all_errors_def {
    use std::collections::BTreeMap;

    use json_resp::CombineErrors;
    use utoipa::{
        openapi::{RefOr, Response, ResponsesBuilder},
        IntoResponses,
    };

    use super::errors::ExtractionErrorOai::*;

    pub struct AllExtErrors;

    impl IntoResponses for AllExtErrors {
        fn responses() -> BTreeMap<String, RefOr<Response>> {
            ResponsesBuilder::new()
                .responses_from_into_responses::<CombineErrors<UnknownError, FailedToBuffer>>()
                .responses_from_into_responses::<LengthLimit>()
                .responses_from_into_responses::<InvalidContentType>()
                .responses_from_into_responses::<FailedToDeserialize>()
                .build()
                .into()
        }
    }

    pub struct PathErrors;

    impl IntoResponses for PathErrors {
        fn responses() -> BTreeMap<String, RefOr<Response>> {
            ResponsesBuilder::new()
                .responses_from_into_responses::<UnknownError>()
                .responses_from_into_responses::<FailedToDeserialize>()
                .build()
                .into()
        }
    }

    pub struct QueryErrors;

    impl IntoResponses for QueryErrors {
        fn responses() -> BTreeMap<String, RefOr<Response>> {
            ResponsesBuilder::new()
                .responses_from_into_responses::<FailedToDeserialize>()
                .build()
                .into()
        }
    }

    pub struct HeaderErrors;

    impl IntoResponses for HeaderErrors {
        fn responses() -> BTreeMap<String, RefOr<Response>> {
            ResponsesBuilder::new()
                .responses_from_into_responses::<UnknownError>()
                .responses_from_into_responses::<FailedToDeserialize>()
                .build()
                .into()
        }
    }
}

pub mod oai {
    use super::errors;

    pub use super::all_errors_def::{AllExtErrors, HeaderErrors, PathErrors, QueryErrors};

    #[allow(non_snake_case)]
    pub mod ExtractionErrorOai {
        /// Error status 422
        ///
        /// For json, path, form, header, query
        pub use super::errors::ExtractionErrorOai::FailedToDeserialize;

        /// Error status 415
        ///
        /// For json, form
        pub use super::errors::ExtractionErrorOai::InvalidContentType;

        /// Error status 413
        ///
        /// For json, form
        pub use super::errors::ExtractionErrorOai::LengthLimit;

        /// Error status 400
        ///
        /// For json, form
        pub use super::errors::ExtractionErrorOai::FailedToBuffer;

        /// Error status 400
        ///
        /// For json, path, form, header
        pub use super::errors::ExtractionErrorOai::UnknownError;

        /// Error status 500
        ///
        /// For path
        pub use super::errors::ExtractionErrorOai::InternalError;
    }
}
