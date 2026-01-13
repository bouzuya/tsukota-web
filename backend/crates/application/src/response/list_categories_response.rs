use crate::view::CategoryView;
use crate::view::PaginatedList;

/// Response for listing categories
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListCategoriesResponse(pub PaginatedList<CategoryView>);
