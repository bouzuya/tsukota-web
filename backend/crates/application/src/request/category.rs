use serde::Deserialize;
use serde::Serialize;

/// Request to add a category
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddCategoryRequest {
    pub name: String,
}

/// Request to update a category
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateCategoryRequest {
    pub name: String,
}
