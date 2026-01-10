use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::repository::EventStoreRepository;
use crate::request::AddCategoryRequest;
use crate::response::AddCategoryResponse;

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
        _user_id: &UserId,
        request: AddCategoryRequest,
    ) -> Result<AddCategoryResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

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
        self.repository.save_events(&account_id, new_events).await?;

        Ok(AddCategoryResponse {
            category_id: category_id.to_string(),
        })
    }
}
