use async_trait::async_trait;
use domain::account::AccountEvent;
use domain::account::AccountId;
use domain::DeviceEvent;
use domain::DeviceId;

use crate::error::ApplicationError;

/// Account repository trait for persisting and loading events
#[async_trait]
pub trait AccountRepository: Send + Sync {
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

/// Device repository trait for persisting and loading events
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    /// Load all events for a given device
    async fn load_events(
        &self,
        device_id: &DeviceId,
    ) -> Result<Vec<DeviceEvent>, ApplicationError>;

    /// Save new events for a given device
    async fn save_events(
        &self,
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), ApplicationError>;
}
