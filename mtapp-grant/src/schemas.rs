use mtapp::Uuid;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrantCreate {
    pub(crate) user_id: Uuid,
    pub(crate) scope_id: Uuid,
}
