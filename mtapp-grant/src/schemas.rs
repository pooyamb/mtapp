use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrantCreate {
    pub(crate) scope_name: String,
}
