use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::RemoveOwnerRequest;
use crate::response::RemoveOwnerResponse;

/// Use case for removing an owner from an account
pub struct RemoveOwnerUseCase<R: AccountRepository> {
    repository: R,
}

impl<R: AccountRepository> RemoveOwnerUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: RemoveOwnerRequest,
    ) -> Result<RemoveOwnerResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Parse user ID to remove
        let owner_id_to_remove: domain::account::UserId = request
            .user_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid user ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::RemoveOwner {
            owner: owner_id_to_remove,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(RemoveOwnerResponse {})
    }
}
