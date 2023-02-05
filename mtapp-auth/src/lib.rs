mod app;
mod errors;
mod extract;
mod handlers;
mod middleware;
mod openapi;
mod providers;
mod schemas;

pub use app::{AuthApp, AuthConfig};
pub use errors::AuthError;
pub use extract::{Claims, TokenBlacklist};
pub use middleware::ClaimCheck;
pub use providers::{GrantProvider, SessionProvider, UserProvider};
pub mod openapi_errors {
    pub use crate::errors::utoipa_response::{
        AuthErrorAuthentication, AuthErrorBadToken, AuthErrorCredentials, AuthErrorPermission,
    };
}
