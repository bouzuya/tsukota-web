/// Request to get an account by ID
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GetAccountRequest {
    pub account_id: String,
}
