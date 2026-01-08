use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::TransactionId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::error::Result;
use crate::repository::EventStoreRepository;
use crate::request::AddTransactionRequest;

/// Use case for adding a transaction to an account
pub struct AddTransactionUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> AddTransactionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        _user_id: &UserId,
        request: AddTransactionRequest,
    ) -> Result<TransactionId> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Generate new transaction ID
        let transaction_id = TransactionId::new();

        // Parse category ID
        let category_id: CategoryId = request
            .category_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid category ID".to_string()))?;

        // Create command
        let command = AccountCommand::AddTransaction {
            transaction_id: transaction_id.clone(),
            amount: request.amount,
            category_id,
            comment: request.comment,
            date: request.date,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(transaction_id)
    }
}
