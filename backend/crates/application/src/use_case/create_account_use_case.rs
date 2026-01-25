use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::CreateAccountRequest;
use crate::response::CreateAccountResponse;

/// Use case for creating a new account
pub struct CreateAccountUseCase<R: AccountRepository> {
    repository: R,
}

impl<R: AccountRepository> CreateAccountUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: &UserId,
        request: CreateAccountRequest,
    ) -> Result<CreateAccountResponse, ApplicationError> {
        // Generate new account ID
        let account_id = AccountId::generate();

        // Create command
        let command = AccountCommand::CreateAccount {
            account_id,
            name: request.name,
            owners: vec![user_id.to_domain()],
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
