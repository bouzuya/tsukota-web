use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::TransactionProjection;
use crate::view::TransactionList;

/// Use case for listing transactions with cursor-based pagination
pub struct ListTransactionsUseCase<A: AccountProjection, T: TransactionProjection> {
    account_projection: A,
    transaction_projection: T,
}

impl<A: AccountProjection, T: TransactionProjection> ListTransactionsUseCase<A, T> {
    pub fn new(account_projection: A, transaction_projection: T) -> Self {
        Self {
            account_projection,
            transaction_projection,
        }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        user_id: &UserId,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<TransactionList, ApplicationError> {
        // Get account to verify ownership
        let account = self
            .account_projection
            .get_account(account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        crate::authorization::verify_owner(&account, user_id)?;

        // Get transactions
        self.transaction_projection
            .list_transactions(account_id, cursor, limit)
            .await
    }
}
