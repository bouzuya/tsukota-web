use domain::account::AccountError;

/// Application layer errors
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Domain error: {0}")]
    Domain(#[from] AccountError),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T> = std::result::Result<T, ApplicationError>;
