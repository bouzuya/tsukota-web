use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;
use domain::TransactionId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::DeleteTransactionRequest;
use crate::response::DeleteTransactionResponse;

/// Use case for deleting a transaction
#[derive(Clone)]
pub struct DeleteTransactionUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl DeleteTransactionUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: DeleteTransactionRequest,
    ) -> Result<DeleteTransactionResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Parse transaction ID
        let transaction_id: TransactionId = request
            .transaction_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid transaction ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::DeleteTransaction { transaction_id };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(DeleteTransactionResponse {})
    }
}
