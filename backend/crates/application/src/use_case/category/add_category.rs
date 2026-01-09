use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::AddCategoryRequest;

/// Use case for adding a category to an account
pub struct AddCategoryUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> AddCategoryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        _user_id: &UserId,
        request: AddCategoryRequest,
    ) -> Result<CategoryId, ApplicationError> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Generate new category ID
        let category_id = CategoryId::new();

        // Create command
        let command = AccountCommand::AddCategory {
            category_id: category_id.clone(),
            name: request.name,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(category_id)
    }
}
