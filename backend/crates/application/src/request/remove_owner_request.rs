/// Request to remove an owner from an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct RemoveOwnerRequest {
    pub account_id: String,
    pub user_id: String,
}
