use std::sync::Arc;

use domain::Account;
use domain::AccountCommand;
use domain::AccountId;
use domain::CategoryId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::repository::AccountRepository;
use crate::request::DeleteCategoryRequest;
use crate::response::DeleteCategoryResponse;

/// Use case for deleting (soft delete) a category
#[derive(Clone)]
pub struct DeleteCategoryUseCase {
    repository: Arc<dyn AccountRepository>,
}

impl DeleteCategoryUseCase {
    pub fn new(repository: Arc<dyn AccountRepository>) -> Self {
        Self { repository }
    }

    #[tracing::instrument(name = "delete_category", skip(self))]
    pub async fn execute(
        &self,
        _user_id: &UserId,
        request: DeleteCategoryRequest,
    ) -> Result<DeleteCategoryResponse, ApplicationError> {
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
        let command = AccountCommand::DeleteCategory { category_id };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository
            .save_events(&account_id, new_events, &aggregate)
            .await?;

        Ok(DeleteCategoryResponse {})
    }
}
