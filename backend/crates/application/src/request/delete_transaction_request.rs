/// Request to delete a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DeleteTransactionRequest {
    pub account_id: String,
    pub transaction_id: String,
}
