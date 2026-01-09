use async_trait::async_trait;
use domain::account::AccountEvent;
use domain::account::AccountId;

use crate::error::ApplicationError;

/// Event store repository trait for persisting and loading events
#[async_trait]
pub trait EventStoreRepository: Send + Sync {
    /// Load all events for a given account
    async fn load_events(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<AccountEvent>, ApplicationError>;

    /// Save new events for a given account
    async fn save_events(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
    ) -> Result<(), ApplicationError>;
}
