use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::TransactionId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::UpdateTransactionRequest;

/// Use case for updating a transaction
pub struct UpdateTransactionUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> UpdateTransactionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: UpdateTransactionRequest,
    ) -> Result<(), ApplicationError> {
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

        // Parse category ID
        let category_id: CategoryId = request
            .category_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid category ID".to_string()))?;

        // Create command
        let command = AccountCommand::UpdateTransaction {
            transaction_id,
            amount: request.amount,
            category_id,
            comment: request.comment,
            date: request.date,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(())
    }
}
