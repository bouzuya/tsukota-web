use serde::Deserialize;
use serde::Serialize;

/// Transaction view model for API responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionView {
    pub id: String,
    pub account_id: String,
    pub amount: String,
    pub category_id: String,
    pub date: String,
    pub comment: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Transaction list with pagination support
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionList {
    pub transactions: Vec<TransactionView>,
    pub next_cursor: Option<String>,
}
