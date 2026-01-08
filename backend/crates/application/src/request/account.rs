use serde::Deserialize;
use serde::Serialize;

/// Request to create a new account
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub owner_id: String,
}

/// Request to update an account
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateAccountRequest {
    pub name: String,
}

/// Request to add an owner to an account
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddOwnerRequest {
    pub user_id: String,
}
