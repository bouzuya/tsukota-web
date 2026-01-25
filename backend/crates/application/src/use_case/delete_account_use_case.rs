use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::DeleteAccountRequest;
use crate::response::DeleteAccountResponse;

/// Use case for deleting an account
pub struct DeleteAccountUseCase<R: AccountRepository> {
    repository: R,
}

impl<R: AccountRepository> DeleteAccountUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: DeleteAccountRequest,
    ) -> Result<DeleteAccountResponse, ApplicationError> {
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
        let command = AccountCommand::DeleteAccount;

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(DeleteAccountResponse {})
    }
}
