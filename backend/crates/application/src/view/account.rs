use serde::Deserialize;
use serde::Serialize;

/// Account view model for API responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub owner_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
