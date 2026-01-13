use crate::view::AccountView;

/// Response for listing accounts
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountView>,
}
