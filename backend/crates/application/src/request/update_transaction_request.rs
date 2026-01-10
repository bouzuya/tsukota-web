/// Request to update a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateTransactionRequest {
    pub account_id: String,
    pub transaction_id: String,
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}
