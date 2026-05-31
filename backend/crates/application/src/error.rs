use domain::AccountError;

/// Google アカウント関連のエラー
#[derive(Debug, thiserror::Error)]
pub enum GoogleUserError {
    /// signup で既に登録済みの Google アカウント
    #[error("Google user already registered")]
    AlreadyRegistered,
    /// signin に対応する Google アカウントが未登録
    #[error("Google user not registered")]
    NotRegistered,
}

/// Application layer errors
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Domain error: {0}")]
    Domain(#[from] AccountError),

    #[error("Google user error: {0}")]
    GoogleUser(#[source] GoogleUserError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("User error: {0}")]
    User(#[source] domain::UserError),
}
