/// Request to add a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddTransactionRequest {
    pub account_id: String,
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}
