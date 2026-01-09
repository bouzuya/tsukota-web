use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;

/// Use case for removing an owner from an account
pub struct RemoveOwnerUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> RemoveOwnerUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        owner_id_to_remove: &UserId,
        _requesting_user_id: &UserId,
    ) -> Result<(), ApplicationError> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::RemoveOwner {
            owner: owner_id_to_remove.clone(),
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
