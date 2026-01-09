/// Request to create a new account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub owner_id: String,
}

/// Request to update an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateAccountRequest {
    pub name: String,
}

/// Request to add an owner to an account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AddOwnerRequest {
    pub user_id: String,
}
