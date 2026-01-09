/// Category view model for API responses
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CategoryView {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}
