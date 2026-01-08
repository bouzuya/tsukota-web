use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::error::Result;
use crate::repository::EventStoreRepository;
use crate::request::AddOwnerRequest;

/// Use case for adding an owner to an account
pub struct AddOwnerUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> AddOwnerUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        _requesting_user_id: &UserId,
        request: AddOwnerRequest,
    ) -> Result<()> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Parse user ID
        let user_id: UserId = request
            .user_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid user ID".to_string()))?;

        // Create command
        let command = AccountCommand::AddOwner { owner: user_id };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
