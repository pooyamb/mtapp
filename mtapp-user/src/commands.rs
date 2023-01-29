use dialoguer::{Input, Password};
use sqlx::PgPool;

use crate::{models::User, schemas::UserCreate};

pub async fn create_user(
    pool: PgPool,
    recv_username: Option<String>,
    recv_password: Option<String>,
) {
    let username = loop {
        let username = if let Some(ref username) = recv_username {
            username.to_owned()
        } else {
            Input::new()
                .with_prompt("Username")
                .interact_text()
                .expect("Failed IO")
        };

        let exist = sqlx::query!(
            "SELECT COUNT(*) FROM users WHERE username = $1",
            recv_username
        )
        .fetch_one(&pool)
        .await
        .expect("Database connection failed!")
        .count
        .unwrap_or(0);

        if exist == 0 {
            break username;
        } else {
            println!("Username already exist");
        }
    };

    let password = if let Some(password) = recv_password {
        password
    } else {
        Password::new()
            .with_prompt("Password")
            .interact()
            .expect("Failed IO")
    };

    User::create(
        UserCreate {
            username,
            email: None,
            password,
        },
        &pool,
    )
    .await
    .expect("User creation failed");

    println!("User created successfully!")
}
