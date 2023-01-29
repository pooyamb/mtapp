use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TokenData {
    pub access_token: String,
    pub token_type: &'static str,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
