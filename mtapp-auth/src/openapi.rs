use json_response::{JsonError, JsonResponse};
use utoipa::{
    openapi::security::{Flow, OAuth2, Password, Scopes, SecurityScheme},
    OpenApi,
};

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

pub(crate) fn get_open_api(path: &str) -> utoipa::openapi::OpenApi {
    let mut openapi = AuthOpenApi::openapi();
    let mut components = openapi.components.unwrap_or_default();
    components.add_security_scheme(
        "jwt_token",
        SecurityScheme::OAuth2(OAuth2::new([Flow::Password(Password::with_refresh_url(
            format!("{}/login?flat=true", path),
            Scopes::from_iter([
                ("superadmin", "Super Admin"),
                ("admin", "Admin"),
                ("confirmed", "Confirmed"),
                ("active", "Active"),
            ]),
            format!("{}/refresh?flat=true", path),
        ))])),
    );
    openapi.components = Some(components);
    openapi
}
