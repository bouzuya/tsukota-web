use std::sync::Arc;

use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::TransactionProjection;
use crate::request::ListTransactionsRequest;
use crate::response::ListTransactionsResponse;

/// Use case for listing transactions with cursor-based pagination
#[derive(Clone)]
pub struct ListTransactionsUseCase {
    account_projection: Arc<dyn AccountProjection>,
    transaction_projection: Arc<dyn TransactionProjection>,
}

impl ListTransactionsUseCase {
    pub fn new(
        account_projection: Arc<dyn AccountProjection>,
        transaction_projection: Arc<dyn TransactionProjection>,
    ) -> Self {
        Self {
            account_projection,
            transaction_projection,
        }
    }

    #[tracing::instrument(name = "list_transactions", skip(self))]
    pub async fn execute(
        &self,
        user_id: &UserId,
        request: ListTransactionsRequest,
    ) -> Result<ListTransactionsResponse, ApplicationError> {
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".into()))?;
        let domain_user_id = user_id.to_domain();

        // Get account to verify ownership
        let account = self
            .account_projection
            .get_account(&account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        crate::authorization::verify_owner(&account, &domain_user_id)?;

        // Get transactions
        let result = self
            .transaction_projection
            .list_transactions(&account_id, request.cursor, request.limit)
            .await?;

        Ok(ListTransactionsResponse(result))
    }
}
