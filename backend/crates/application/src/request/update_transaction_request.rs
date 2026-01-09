/// Request to update a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateTransactionRequest {
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}
