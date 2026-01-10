/// Request to list transactions with cursor-based pagination
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListTransactionsRequest {
    pub account_id: String,
    pub cursor: Option<String>,
    pub limit: usize,
}
