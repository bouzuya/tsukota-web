use domain::account::Account;
use domain::account::AccountCommand;
use domain::account::AccountId;
use domain::account::CategoryId;
use domain::account::UserId;

use crate::error::Result;
use crate::repository::EventStoreRepository;

/// Use case for deleting (soft delete) a category
pub struct DeleteCategoryUseCase<R: EventStoreRepository> {
    repository: R,
}

impl<R: EventStoreRepository> DeleteCategoryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        category_id: &CategoryId,
        _user_id: &UserId,
    ) -> Result<()> {
        // Load events
        let events = self.repository.load_events(account_id).await?;

        // Reconstruct aggregate
        let aggregate = Account::from_events(events);

        // Create command
        let command = AccountCommand::DeleteCategory {
            category_id: category_id.clone(),
        };

        // Handle command
        let new_events = aggregate.handle_command(command)?;

        // Save events
        self.repository.save_events(account_id, new_events).await?;

        Ok(())
    }
}
