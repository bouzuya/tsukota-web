/// Response for adding a category
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddCategoryResponse {
    pub category_id: String,
}
