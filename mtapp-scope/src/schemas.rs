use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ScopeCreate {
    pub name: String,
}
