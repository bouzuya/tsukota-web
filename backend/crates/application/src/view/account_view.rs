/// Account view model for API responses
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AccountView {
    pub id: String,
    pub name: String,
    pub owner_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
