use mtapp::Uuid;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GrantCreate {
    pub(crate) user_id: Uuid,
    pub(crate) scope_id: Uuid,
}
