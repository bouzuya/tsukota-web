use super::TransactionView;

/// Transaction list with pagination support
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct TransactionList {
    pub transactions: Vec<TransactionView>,
    pub next_cursor: Option<String>,
}
