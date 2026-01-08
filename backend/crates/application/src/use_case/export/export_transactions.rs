use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::error::Result;
use crate::projection::AccountProjection;
use crate::projection::TransactionProjection;
use crate::view::TransactionView;

/// Use case for exporting transactions as JSON for a specific month
pub struct ExportTransactionsUseCase<A: AccountProjection, T: TransactionProjection> {
    account_projection: A,
    transaction_projection: T,
}

impl<A: AccountProjection, T: TransactionProjection> ExportTransactionsUseCase<A, T> {
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
        year: i32,
        month: u32,
    ) -> Result<Vec<TransactionView>> {
        // Validate month
        if !(1..=12).contains(&month) {
            return Err(ApplicationError::InvalidRequest(format!(
                "Invalid month: {}. Must be between 1 and 12",
                month
            )));
        }

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

        // Get transactions for the specified month
        self.transaction_projection
            .list_transactions_for_month(account_id, year, month)
            .await
    }
}
