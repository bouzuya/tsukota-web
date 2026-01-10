/// Request to delete an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DeleteAccountRequest {
    pub account_id: String,
}
