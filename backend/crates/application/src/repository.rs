use async_trait::async_trait;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;
use domain::GoogleUserId;
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

/// Google sub と内部 UserId の対応を管理するサイドインデックス
#[async_trait]
pub trait GoogleUserMapRepository: Send + Sync {
    /// Google の sub から内部 UserId を引く。未登録は Ok(None)
    async fn find_user_id_by_google_user_id(
        &self,
        google_user_id: &GoogleUserId,
    ) -> Result<Option<UserId>, ApplicationError>;

    /// Google の sub と内部 UserId の対応を新規登録する。
    /// 既存の場合は ApplicationError::GoogleUser(GoogleUserError::AlreadyRegistered) を返す
    async fn save(
        &self,
        google_user_id: &GoogleUserId,
        user_id: &UserId,
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
