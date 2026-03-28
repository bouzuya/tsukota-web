use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::UpdateAccountRequest;
use crate::response::UpdateAccountResponse;

/// Use case for updating account details
#[derive(Clone)]
pub struct UpdateAccountUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl UpdateAccountUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "update_account", skip(self))]
    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: UpdateAccountRequest,
    ) -> Result<UpdateAccountResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command (no user_id needed - authorization done in application layer)
        let command = AccountCommand::UpdateAccount { name: request.name };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(UpdateAccountResponse {})
    }
}
