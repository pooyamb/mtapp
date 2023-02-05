use dialoguer::MultiSelect;
use sqlx::PgPool;

use mtapp_scope::Scope;
use mtapp_user::User;

use crate::{models::Grant, schemas::GrantCreate};

pub async fn manage_grants(pool: PgPool, recv_username: String) {
    let user = User::get_by_username(&recv_username, &pool)
        .await
        .expect("Failed to retrieve user.");

    let scopes = Scope::find(&Default::default(), &pool)
        .await
        .expect("Failed to retrieve scopes.");

    let grants = Grant::find_for_user(user.id, &pool)
        .await
        .expect("Failed to assign scopes");

    let marked_scopes = scopes
        .into_iter()
        .map(|r| {
            if grants.iter().any(|g| g == &r.name) {
                (r, true)
            } else {
                (r, false)
            }
        })
        .collect::<Vec<_>>();

    let selected_scopes = MultiSelect::new()
        .with_prompt("Please select the scopes you want to assign")
        .items_checked(
            &marked_scopes
                .iter()
                .map(|r| (r.0.name.as_str(), r.1))
                .collect::<Vec<_>>(),
        )
        .interact()
        .expect("Failed IO");

    let mut tx = pool
        .begin()
        .await
        .expect("Failed to start a database transaction");

    for (idx, (scope, granted)) in marked_scopes.iter().enumerate() {
        if selected_scopes.contains(&idx) && !granted {
            Grant::create(
                GrantCreate {
                    user_id: user.id,
                    scope_id: scope.id,
                },
                &mut tx,
            )
            .await
            .expect("Failed to assign scopes");
        } else if !selected_scopes.contains(&idx) && *granted {
            Grant::delete_by_ids(user.id, scope.id, &mut tx)
                .await
                .expect("Failed to assign scopes");
        }
    }

    tx.commit().await.expect("User creation failed");

    println!("User created successfully!")
}
