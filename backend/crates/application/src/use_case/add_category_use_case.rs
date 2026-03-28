use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;
use domain::CategoryId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::AddCategoryRequest;
use crate::response::AddCategoryResponse;

/// Use case for adding a category to an account
#[derive(Clone)]
pub struct AddCategoryUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl AddCategoryUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "add_category", skip(self))]
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
        let category_id = CategoryId::generate();

        // Create command
        let command = AccountCommand::AddCategory {
            category_id,
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
