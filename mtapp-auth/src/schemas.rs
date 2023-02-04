use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: &'static str,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
