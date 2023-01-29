use dialoguer::MultiSelect;
use sqlx::PgPool;

use mtapp_scope::Scope;
use mtapp_user::User;

use crate::models::Grant;

pub async fn manage_grants(pool: PgPool, recv_username: String) {
    let user = User::get_by_username(&recv_username, &pool)
        .await
        .expect("Failed to retrieve user.");

    let scopes = Scope::find(&Default::default(), &pool)
        .await
        .expect("Failed to retrieve scopes.");

    let grants = Grant::get_grants(user.id, &pool)
        .await
        .expect("Failed to assign scopes");

    let marked_scopes = scopes
        .into_iter()
        .map(|r| {
            if grants.iter().any(|g| g.scope_id == r.id) {
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
            Grant::add_grant(user.id, &scope.name, &mut tx)
                .await
                .expect("Failed to assign scopes");
        }
        if !selected_scopes.contains(&idx) && *granted {
            Grant::del_grant(user.id, &scope.name, &mut tx)
                .await
                .expect("Failed to assign scopes");
        }
    }

    tx.commit().await.expect("User creation failed");

    println!("User created successfully!")
}
