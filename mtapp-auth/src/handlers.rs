use axum::{
    http::header::SET_COOKIE,
    response::{AppendHeaders, IntoResponse},
    Extension, Form,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use basteh::Storage;
use json_response::{InternalErrorResponse, JsonResponse};

use crate::{
    app::AuthConfig,
    errors::utoipa_response::{AuthErrorBadToken, AuthErrorCredentials, AuthErrorPermission},
    errors::AuthError,
    extract::Claims,
    providers::{GrantProvider, SessionProvider, UserProvider},
    schemas::{Credentials, TokenData},
};

#[utoipa::path(
    post,
    tag = "Auth",
    path = "/login",
    request_body(
        content=inline(Credentials),
        content_type="application/x-www-form-urlencoded",
        description="User credentials"
    ),
    responses(
        (
            status = 200,
            body=TokenData,
            description="Login was successful",
            headers(
                (
                    "Set-Cookie" = String,
                    description="Contains a cookie name `refresh-token` which is used \
                        by both the `/refresh` endpoint and the `/logout` endpoint"
                )
            )
        ),
        (status = 401, response=AuthErrorCredentials),
        (status = 403, response=AuthErrorPermission),
        (status = 500, response=InternalErrorResponse),
    )
)]
pub async fn login<U, S, G>(
    config: Extension<AuthConfig>,
    user_data: U::Data<()>,
    session_data: S::Data<()>,
    scopes_data: G::Data<()>,
    credentials: Form<Credentials>,
) -> impl IntoResponse
where
    U: UserProvider,
    S: SessionProvider,
    G: GrantProvider,
{
    let user_id = U::login(&user_data, &credentials.username, &credentials.password).await?;
    let scopes = G::scopes(&scopes_data, user_id).await?;

    let (jti, refresh_token) = S::make(&session_data, user_id).await?;

    let claims = Claims::new(user_id, jti, scopes, config.get_token_expiry());
    let access_token = claims.generate_token(config.expose_secret());

    let headers = AppendHeaders([(
        SET_COOKIE,
        Cookie::build("refresh-token", &refresh_token)
            .http_only(true)
            .finish()
            .stripped()
            .to_string(),
    )]);

    let json = JsonResponse::with_content(TokenData {
        access_token,
        token_type: "bearer",
        refresh_token,
        expires_in: config.get_token_expiry().as_secs(),
    });

    Result::<_, AuthError>::Ok((headers, json))
}

#[utoipa::path(
    post,
    tag = "Auth",
    path = "/refresh",
    params(
        ("refresh-token" = Option<String>, Cookie, description = "Refresh token")
    ),
    responses(
        (
            status = 200, 
            body=TokenData, 
            headers(
                (
                    "Set-Cookie" = String,
                    description="Contains a cookie name `refresh-token` which is used \
                        by both the `/refresh` endpoint and the `/logout` endpoint"
                )
            )
        ),
        (status = 401, response=AuthErrorBadToken),
        (status = 403, response=AuthErrorPermission),
        (status = 500, response=InternalErrorResponse),
    )
)]
pub async fn refresh<S, G>(
    config: Extension<AuthConfig>,
    storage: Extension<Storage>,
    session_data: S::Data<()>,
    grants_data: G::Data<()>,
    cookies: CookieJar,
) -> impl IntoResponse
where
    S: SessionProvider,
    G: GrantProvider,
{
    let refresh_token = if let Some(cookie) = cookies.get("refresh-token") {
        String::from(cookie.value())
    } else {
        return Err(AuthError::BadToken);
    };

    let (jti, user_id) = S::find(&session_data, &refresh_token).await?;

    let scopes = G::scopes(&grants_data, user_id).await?;

    // Blacklist the previous jti
    storage
        .scope(config.blacklist_scope())
        .set_expiring(jti, b"", config.get_token_expiry())
        .await?;

    let jti = S::reset_jti(&session_data, &refresh_token).await?;

    let claims = Claims::new(user_id, jti, scopes, config.get_token_expiry());
    let access_token = claims.generate_token(config.expose_secret());

    let jr = JsonResponse::with_content(TokenData {
        access_token,
        token_type: "bearer",
        refresh_token,
        expires_in: config.get_token_expiry().as_secs(),
    });

    Ok(jr)
}

#[utoipa::path(
    post,
    tag = "Auth",
    path = "/logout",
    params(
        ("refresh-token" = Option<String>, Cookie, description = "Refresh token")
    ),
    responses(
        (status = 200, body=TokenData),
        (status = 401, response=AuthErrorBadToken),
        (status = 403, response=AuthErrorPermission),
        (status = 500, response=InternalErrorResponse),
    )
)]
pub async fn logout<U, S>(
    config: Extension<AuthConfig>,
    storage: Extension<Storage>,
    claims: Option<Extension<Claims>>,
    cookies: CookieJar,
    session_data: S::Data<()>,
) -> impl IntoResponse
where
    U: UserProvider,
    S: SessionProvider,
{
    if let Some(claims) = claims {
        S::delete_by_jti(&session_data, claims.jti).await?;

        // Blacklist the previous jti
        storage
            .scope(config.blacklist_scope())
            .set_expiring(claims.jti, b"", config.get_token_expiry())
            .await?;
    } else if let Some(cookie) = cookies.get("refresh-token") {
        let (jti, _) = S::find(&session_data, cookie.value()).await?;

        // Blacklist the previous jti
        storage
            .scope(config.blacklist_scope())
            .set_expiring(jti, b"", config.get_token_expiry())
            .await?;

        S::delete_by_jti(&session_data, jti).await?;
    }

    Result::<_, AuthError>::Ok(JsonResponse::with_content("Logged out successfully"))
}
