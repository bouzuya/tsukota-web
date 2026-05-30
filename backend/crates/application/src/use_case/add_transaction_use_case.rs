use std::error::Error;
use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;
use domain::CategoryId;
use domain::TransactionId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::AddTransactionRequest;
use crate::response::AddTransactionResponse;

#[derive(Debug, thiserror::Error)]
enum E {
    #[error("failed to handle command")]
    HandleCommand(#[source] domain::AccountError),
    #[error("invalid account id")]
    InvalidAccountId(#[source] domain::AccountIdError),
    #[error("invalid category id")]
    InvalidCategoryId(#[source] domain::CategoryIdError),
    #[error("load events")]
    LoadEvents(#[source] ApplicationError),
    #[error("save events")]
    SaveEvents(#[source] ApplicationError),
}

impl From<E> for ApplicationError {
    fn from(err: E) -> Self {
        // FIXME
        ApplicationError::InvalidRequest(err.to_string())
    }
}

/// Use case for adding a transaction to an account
#[derive(Clone)]
pub struct AddTransactionUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl AddTransactionUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "add_transaction", skip(self))]
    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: AddTransactionRequest,
    ) -> Result<AddTransactionResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request.account_id.parse().map_err(E::InvalidAccountId)?;

        // Load events
        let events = self
            .repository
            .load_events(&account_id)
            .await
            .map_err(E::LoadEvents)?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Generate new transaction ID
        let transaction_id = TransactionId::generate();

        // Parse category ID
        let category_id: CategoryId = request.category_id.parse().map_err(E::InvalidCategoryId)?;

        // Create command
        let command = AccountCommand::AddTransaction {
            transaction_id,
            amount: request.amount,
            category_id,
            comment: request.comment,
            date: request.date,
        };

        // Handle command
        let new_events = aggregate
            .handle_command(command)
            .map_err(E::HandleCommand)?;

        // Save events
        self.repository
            .save_events(&account_id, new_events, &aggregate)
            .await
            .map_err(E::SaveEvents)?;

        Ok(AddTransactionResponse {
            transaction_id: transaction_id.to_string(),
        })
    }
}
