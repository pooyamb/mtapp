use utoipa::{
    openapi::{ComponentsBuilder, InfoBuilder, OpenApi, OpenApiBuilder},
    ToSchema,
};

use crate::extractors::oai::ExtractionErrorOai;

macro_rules! pack_errors {
    ($($error:path),*) => {
        [
            $((<$error as ToSchema>::schema()),)*
        ]
    };
}

pub(crate) fn generate_openapi(description: &str, version: &str) -> OpenApi {
    let errors = pack_errors![
        ExtractionErrorOai::InvalidContentType,
        ExtractionErrorOai::FailedToBuffer,
        ExtractionErrorOai::FailedToDeserialize,
        ExtractionErrorOai::LengthLimit,
        ExtractionErrorOai::UnknownError
    ];

    let mut components = ComponentsBuilder::new();

    for err in errors {
        components = components.schema(err.0, err.1);
    }

    OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .description(Some(description))
                .version(version)
                .build(),
        )
        .components(Some(components.build()))
        .build()
}
