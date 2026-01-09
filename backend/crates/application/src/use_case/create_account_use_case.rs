use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::CreateAccountRequest;

/// Use case for creating a new account
pub struct CreateAccountUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> CreateAccountUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        request: CreateAccountRequest,
    ) -> Result<AccountId, ApplicationError> {
        // Generate new account ID
        let account_id = AccountId::new();

        // Parse owner ID
        let owner_id: UserId = request
            .owner_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid owner ID".to_string()))?;

        // Create command
        let command = AccountCommand::CreateAccount {
            account_id: account_id.clone(),
            name: request.name,
            owners: vec![owner_id],
        };

        // Handle command on empty aggregate
        let aggregate = Account::Empty;
        let events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, events).await?;

        Ok(account_id)
    }
}
