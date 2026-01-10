/// Request to list all categories of an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListCategoriesRequest {
    pub account_id: String,
}
