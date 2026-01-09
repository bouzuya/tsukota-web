/// Request to add an owner to an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddOwnerRequest {
    pub user_id: String,
}
