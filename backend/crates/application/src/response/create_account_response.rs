/// Response for creating a new account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateAccountResponse {
    pub account_id: String,
}
