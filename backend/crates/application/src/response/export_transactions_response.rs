use crate::view::TransactionView;

/// Response for exporting transactions
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ExportTransactionsResponse {
    pub transactions: Vec<TransactionView>,
}
