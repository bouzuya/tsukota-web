/// Request to create a new account
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateAccountRequest {
    pub name: String,
}
