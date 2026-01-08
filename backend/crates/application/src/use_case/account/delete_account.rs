use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::UserId;

use crate::error::Result;
use crate::repository::EventStoreRepository;

/// Use case for deleting an account
pub struct DeleteAccountUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> DeleteAccountUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, account_id: &AccountId, _user_id: &UserId) -> Result<()> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command (no user_id needed - authorization done in application layer)
        let command = AccountCommand::DeleteAccount;

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
