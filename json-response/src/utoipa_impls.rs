use utoipa::{
    openapi::{
        ContentBuilder, KnownFormat, ObjectBuilder, Ref, RefOr, Response, ResponseBuilder,
        ResponsesBuilder, Schema, SchemaFormat, SchemaType,
    },
    IntoResponses, ToResponse, ToSchema,
};

use crate::{JsonError, JsonResponse};

impl<T, M> ToSchema<'static> for JsonResponse<T, M>
where
    T: ToSchema<'static>,
    M: ToSchema<'static>,
{
    fn schema() -> (&'static str, RefOr<Schema>) {
        let inner_schema = T::schema();
        (
            inner_schema.0,
            ObjectBuilder::new()
                .property(
                    "status",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Integer)
                        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32))),
                )
                .required("status")
                .property("content", inner_schema.1)
                .required("content")
                .property("meta", M::schema().1)
                .required("meta")
                .into(),
        )
    }
}

impl<T> ToSchema<'static> for JsonError<T>
where
    T: ToSchema<'static>,
{
    fn schema() -> (&'static str, RefOr<Schema>) {
        (
            "JsonError",
            ObjectBuilder::new()
                .property(
                    "status",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::Integer)
                        .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32))),
                )
                .required("status")
                .property(
                    "code",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::String)
                        .format(Some(SchemaFormat::Custom(String::from(
                            "([0-9]*) ([a-zA-Z\\-]*)",
                        )))),
                )
                .required("code")
                .property(
                    "hint",
                    ObjectBuilder::new()
                        .schema_type(SchemaType::String)
                        .nullable(true),
                )
                .property("content", T::schema().1)
                .required("content")
                .into(),
        )
    }
}

/// This struct is only meant to be used for utoipa
pub struct InternalErrorResponse;

impl ToResponse<'static> for InternalErrorResponse {
    fn response() -> (&'static str, RefOr<Response>) {
        (
            "InternalError",
            ResponseBuilder::new()
                .content(
                    "application/json",
                    ContentBuilder::new()
                        .schema(
                            ObjectBuilder::new()
                                .property(
                                    "status",
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::Integer)
                                        .enum_values(Some([500]))
                                        .example(Some(500.into())),
                                )
                                .property(
                                    "code",
                                    ObjectBuilder::new()
                                        .schema_type(SchemaType::String)
                                        .enum_values(Some(["50000 internal-error"]))
                                        .example(Some("50000 internal-error".into())),
                                )
                                .build(),
                        )
                        .build()
                        .into(),
                )
                .build()
                .into(),
        )
    }
}

impl IntoResponses for InternalErrorResponse {
    fn responses() -> std::collections::BTreeMap<String, RefOr<Response>> {
        ResponsesBuilder::new()
            .response("500", Ref::from_response_name(Self::response().0))
            .build()
            .into()
    }
}
