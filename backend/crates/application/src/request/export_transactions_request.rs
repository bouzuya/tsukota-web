/// Request to export transactions for a specific month
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ExportTransactionsRequest {
    pub account_id: String,
    pub year: i32,
    pub month: u32,
}
