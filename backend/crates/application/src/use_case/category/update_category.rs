use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::UpdateCategoryRequest;

/// Use case for updating a category
pub struct UpdateCategoryUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> UpdateCategoryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        category_id: &CategoryId,
        _user_id: &UserId,
        request: UpdateCategoryRequest,
    ) -> Result<(), ApplicationError> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::UpdateCategory {
            category_id: category_id.clone(),
            name: request.name,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
