/// Request to delete a category
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DeleteCategoryRequest {
    pub account_id: String,
    pub category_id: String,
}
