use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::AddOwnerRequest;
use crate::response::AddOwnerResponse;

/// Use case for adding an owner to an account
#[derive(Clone)]
pub struct AddOwnerUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl AddOwnerUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "add_owner", skip(self))]
    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: AddOwnerRequest,
    ) -> Result<AddOwnerResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Parse user ID to add as owner
        let owner_id: domain::UserId = request
            .user_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid user ID".to_string()))?;

        // Create command
        let command = AccountCommand::AddOwner { owner: owner_id };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(AddOwnerResponse {})
    }
}
