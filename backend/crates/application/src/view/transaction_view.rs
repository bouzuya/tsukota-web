/// Transaction view model for API responses
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
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
