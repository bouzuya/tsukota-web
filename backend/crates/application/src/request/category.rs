/// Request to add a category
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddCategoryRequest {
    pub name: String,
}

/// Request to update a category
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
}
