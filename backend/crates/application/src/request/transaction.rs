/// Request to add a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddTransactionRequest {
    pub amount: String,
    pub category_id: String,
    pub date: String,
    pub comment: String,
}

/// Request to update a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateTransactionRequest {
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}
