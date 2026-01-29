use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::CreateAccountRequest;
use crate::response::CreateAccountResponse;

/// Use case for creating a new account
#[derive(Clone)]
pub struct CreateAccountUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl CreateAccountUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
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
