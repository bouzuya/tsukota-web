use domain::account::AccountError;

/// Application layer errors
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Device error: {0}")]
    Device(#[source] domain::DeviceError),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Domain error: {0}")]
    Domain(#[from] AccountError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("User error: {0}")]
    User(#[source] domain::UserError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}
