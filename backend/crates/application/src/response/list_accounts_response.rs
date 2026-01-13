use crate::view::AccountView;
use crate::view::PaginatedList;

/// Response for listing accounts
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListAccountsResponse(pub PaginatedList<AccountView>);
