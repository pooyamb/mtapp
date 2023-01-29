use actix_storage::Storage;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Extension;
use mtapp_auth::{AuthError, Claims};

use crate::errors::UserError;

pub(crate) async fn user_ban_check<B>(
    storage: Extension<Storage>,
    claims: Option<Extension<Claims>>,
    request: Request<B>,
    next: Next<B>,
) -> axum::response::Response {
    if let Some(claims) = claims {
        match storage
            .scope("banned_user_ids")
            .contains_key(claims.inner().user_id)
            .await
        {
            Ok(res) => {
                if res {
                    return AuthError::Permission.into_response();
                }
            }
            Err(e) => return UserError::Other(Box::new(e)).into_response(),
        }
    }

    next.run(request).await
}
