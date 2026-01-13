use crate::view::CategoryView;

/// Response for listing categories
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListCategoriesResponse {
    pub categories: Vec<CategoryView>,
}
