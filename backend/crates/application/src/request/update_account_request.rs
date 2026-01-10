/// Request to update an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateAccountRequest {
    pub account_id: String,
    pub name: String,
}
