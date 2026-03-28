use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;
use domain::CategoryId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::UpdateCategoryRequest;
use crate::response::UpdateCategoryResponse;

/// Use case for updating a category
#[derive(Clone)]
pub struct UpdateCategoryUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl UpdateCategoryUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "update_category", skip(self))]
    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: UpdateCategoryRequest,
    ) -> Result<UpdateCategoryResponse, ApplicationError> {
        // Parse account ID
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".to_string()))?;

        // Parse category ID
        let category_id: CategoryId = request
            .category_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid category ID".to_string()))?;

        // Load events
        let events = self.repository.load_events(&account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::UpdateCategory {
            category_id,
            name: request.name,
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(&account_id, new_events).await?;

        Ok(UpdateCategoryResponse {})
    }
}
