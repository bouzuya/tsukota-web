use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::CreateAccountRequest;
use crate::response::CreateAccountResponse;

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
        user_id: &UserId,
        request: CreateAccountRequest,
    ) -> Result<CreateAccountResponse, ApplicationError> {
        // Generate new account ID
        let account_id = AccountId::new();

        // Create command
        let command = AccountCommand::CreateAccount {
            account_id: account_id.clone(),
            name: request.name,
            owners: vec![user_id.clone()],
        };

        // Handle command on empty aggregate
        let aggregate = Account::Empty;
        let events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, events).await?;

        Ok(CreateAccountResponse {
            account_id: account_id.to_string(),
        })
    }
}
