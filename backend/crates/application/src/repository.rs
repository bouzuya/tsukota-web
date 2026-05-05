use async_trait::async_trait;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;
use domain::DeviceEvent;
use domain::DeviceId;
use domain::UserEvent;
use domain::UserId;

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
        aggregate: &Account,
    ) -> Result<(), ApplicationError>;
}

/// Device repository trait for persisting and loading events
#[async_trait]
pub trait DeviceRepository: Send + Sync {
    /// Load all events for a given device
    async fn load_events(&self, device_id: &DeviceId)
    -> Result<Vec<DeviceEvent>, ApplicationError>;

    /// Save new events for a given device
    async fn save_events(
        &self,
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), ApplicationError>;
}

/// User repository trait for persisting and loading events
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Load all events for a given user
    async fn load_events(&self, user_id: &UserId) -> Result<Vec<UserEvent>, ApplicationError>;

    /// Save new events for a given user
    async fn save_events(
        &self,
        user_id: &UserId,
        events: Vec<UserEvent>,
    ) -> Result<(), ApplicationError>;
}
