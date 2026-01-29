use std::collections::HashMap;
use std::sync::Arc;

use application::error::ApplicationError;
use application::repository::AccountRepository;
use async_trait::async_trait;
use domain::AccountEvent;
use domain::AccountId;
use tokio::sync::RwLock;

/// In-memory event store for development and testing
#[derive(Clone, Default)]
pub struct InMemoryEventStore {
    events: Arc<RwLock<HashMap<String, Vec<AccountEvent>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the shared event storage for use with projections
    pub fn events(&self) -> Arc<RwLock<HashMap<String, Vec<AccountEvent>>>> {
        self.events.clone()
    }
}

#[async_trait]
impl AccountRepository for InMemoryEventStore {
    async fn load_events(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<AccountEvent>, ApplicationError> {
        let events = self.events.read().await;
        let account_events = events
            .get(&account_id.to_string())
            .cloned()
            .unwrap_or_default();
        Ok(account_events)
    }

    async fn save_events(
        &self,
        account_id: &AccountId,
        new_events: Vec<AccountEvent>,
    ) -> Result<(), ApplicationError> {
        let mut events = self.events.write().await;
        let account_events = events.entry(account_id.to_string()).or_default();
        account_events.extend(new_events);
        Ok(())
    }
}
