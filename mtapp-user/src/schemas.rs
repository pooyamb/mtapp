use serde::Deserialize;
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct UserCreate {
    #[validate(length(min = 4, max = 48))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Validate, Deserialize, Default)]
pub struct UserUpdate {
    #[validate(length(min = 4, max = 48))]
    pub username: Option<String>,
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
}

#[derive(Validate, Deserialize, Default)]
pub struct SelfUpdate {
    #[validate(length(min = 8, max = 128))]
    pub password: Option<String>,
}

impl Into<UserUpdate> for SelfUpdate {
    fn into(self) -> UserUpdate {
        UserUpdate {
            username: None,
            password: self.password,
            email: None,
        }
    }
}

#[derive(Validate, Deserialize)]
pub struct UserRegister {
    #[validate(length(min = 6, max = 48))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(email)]
    pub email: String,
}

impl Into<UserCreate> for UserRegister {
    fn into(self) -> UserCreate {
        UserCreate {
            username: self.username,
            password: self.password,
            email: Some(self.email),
        }
    }
}
