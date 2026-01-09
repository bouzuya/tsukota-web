use async_trait::async_trait;
use domain::account::AccountId;
use domain::account::TransactionId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::view::AccountView;
use crate::view::CategoryView;
use crate::view::TransactionList;
use crate::view::TransactionView;

/// Projection trait for Account read models
#[async_trait]
pub trait AccountProjection: Send + Sync {
    /// Get a single account by ID
    async fn get_account(
        &self,
        account_id: &AccountId,
    ) -> Result<Option<AccountView>, ApplicationError>;

    /// List all accounts owned by a user
    async fn list_accounts(
        &self,
        owner_id: &UserId,
    ) -> Result<Vec<AccountView>, ApplicationError>;
}

/// Projection trait for Category read models
#[async_trait]
pub trait CategoryProjection: Send + Sync {
    /// List all categories for an account (including deleted ones)
    async fn list_categories(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<CategoryView>, ApplicationError>;
}

/// Projection trait for Transaction read models
#[async_trait]
pub trait TransactionProjection: Send + Sync {
    /// List transactions with cursor-based pagination
    async fn list_transactions(
        &self,
        account_id: &AccountId,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<TransactionList, ApplicationError>;

    /// Get a single transaction by ID
    async fn get_transaction(
        &self,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> Result<Option<TransactionView>, ApplicationError>;

    /// List all transactions for a specific month
    async fn list_transactions_for_month(
        &self,
        account_id: &AccountId,
        year: i32,
        month: u32,
    ) -> Result<Vec<TransactionView>, ApplicationError>;
}
