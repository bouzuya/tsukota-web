use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::TransactionId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;

/// Use case for deleting a transaction
pub struct DeleteTransactionUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> DeleteTransactionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        transaction_id: &TransactionId,
        _user_id: &UserId,
    ) -> Result<(), ApplicationError> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::DeleteTransaction {
            transaction_id: transaction_id.clone(),
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
