use std::sync::Arc;

use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::TransactionProjection;
use crate::request::ExportTransactionsRequest;
use crate::response::ExportTransactionsResponse;
use crate::view::PaginatedList;

/// Use case for exporting transactions as JSON for a specific month
#[derive(Clone)]
pub struct ExportTransactionsUseCase {
    account_projection: Arc<dyn AccountProjection>,
    transaction_projection: Arc<dyn TransactionProjection>,
}

impl ExportTransactionsUseCase {
    pub fn new(
        account_projection: Arc<dyn AccountProjection>,
        transaction_projection: Arc<dyn TransactionProjection>,
    ) -> Self {
        Self {
            account_projection,
            transaction_projection,
        }
    }

    #[tracing::instrument(name = "export_transactions", skip(self))]
    pub async fn execute(
        &self,
        user_id: &UserId,
        request: ExportTransactionsRequest,
    ) -> Result<ExportTransactionsResponse, ApplicationError> {
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".into()))?;
        let domain_user_id = user_id.to_domain();

        // Validate month
        if !(1..=12).contains(&request.month) {
            return Err(ApplicationError::InvalidRequest(format!(
                "Invalid month: {}. Must be between 1 and 12",
                request.month
            )));
        }

        // Verify user is an owner of the account
        let owner_ids = self
            .account_projection
            .list_account_owner_ids(&account_id)
            .await?;
        crate::authorization::verify_owner(&account_id, &owner_ids, &domain_user_id)?;

        // Get transactions for the specified month
        let transactions = self
            .transaction_projection
            .list_transactions_for_month(&account_id, request.year, request.month)
            .await?;

        Ok(ExportTransactionsResponse(PaginatedList {
            items: transactions,
            next_cursor: None,
        }))
    }
}
