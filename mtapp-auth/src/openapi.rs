use utoipa::{
    openapi::security::{Flow, OAuth2, Password, Scopes, SecurityScheme},
    OpenApi,
};

use crate::{
    errors::AuthErrorOai,
    handlers::*,
    schemas::{Message, TokenData},
};

#[derive(OpenApi)]
#[openapi(
    paths(login, refresh, logout),
    components(schemas(
        TokenData,
        Message,
        AuthErrorOai::Authentication,
        AuthErrorOai::BadToken,
        AuthErrorOai::Permission,
        AuthErrorOai::Credentials,
        AuthErrorOai::InternalError
    ))
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
