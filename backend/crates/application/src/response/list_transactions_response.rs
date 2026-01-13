use crate::view::TransactionView;

/// Response for listing transactions
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListTransactionsResponse {
    pub transactions: Vec<TransactionView>,
    pub next_cursor: Option<String>,
}
