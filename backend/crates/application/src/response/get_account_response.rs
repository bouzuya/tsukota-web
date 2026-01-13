use crate::view::AccountView;

/// Response for getting account details
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GetAccountResponse {
    pub account: AccountView,
}
