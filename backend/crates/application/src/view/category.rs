use serde::Deserialize;
use serde::Serialize;

/// Category view model for API responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryView {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}
