use crate::view::PaginatedList;
use crate::view::TransactionView;

/// Response for listing transactions
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListTransactionsResponse(pub PaginatedList<TransactionView>);
