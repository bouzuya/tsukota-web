/// Paginated list of transactions
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}
