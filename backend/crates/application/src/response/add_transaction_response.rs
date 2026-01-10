/// Response for adding a transaction
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddTransactionResponse {
    pub transaction_id: String,
}
