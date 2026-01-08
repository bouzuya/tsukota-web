use serde::Deserialize;
use serde::Serialize;

/// Request to add a transaction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddTransactionRequest {
    pub amount: String,
    pub category_id: String,
    pub date: String,
    pub comment: String,
}

/// Request to update a transaction
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateTransactionRequest {
    pub amount: String,
    pub category_id: String,
    pub date: String,
    pub comment: String,
}
