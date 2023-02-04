use json_response::{JsonError, JsonResponse};
use utoipa::OpenApi;

use crate::{
    errors::utoipa_response::{
        AuthErrorAuthentication, AuthErrorBadToken, AuthErrorCredentials, AuthErrorPermission,
    },
    handlers::*,
    schemas::TokenData,
};

type TokenDataJson = JsonResponse<TokenData>;

#[derive(OpenApi)]
#[openapi(
    paths(login, refresh, logout),
    components(
        schemas(TokenDataJson, JsonError),
        responses(
            AuthErrorAuthentication,
            AuthErrorBadToken,
            AuthErrorPermission,
            AuthErrorCredentials
        )
    )
)]
pub(crate) struct AuthOpenApi;
