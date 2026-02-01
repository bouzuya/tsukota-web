use std::collections::BTreeSet;

use crate::AccountId;
use crate::UserCommand;
use crate::UserEvent;
use crate::UserEventCommonProps;
use crate::UserId;

/// ユーザー集約のエラー
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Account already added")]
    AccountAlreadyAdded,
    #[error("Account not found")]
    AccountNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
}

/// ユーザー集約
#[derive(Clone, Debug, PartialEq)]
pub enum User {
    /// ユーザー作成前の空の状態
    Empty,
    /// ユーザー作成後のアクティブな状態
    Active(ActiveUser),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveUser {
    /// アカウント ID のセット
    account_ids: BTreeSet<AccountId>,
    /// ユーザー ID
    id: UserId,
}

impl ActiveUser {
    pub fn account_ids(&self) -> &BTreeSet<AccountId> {
        &self.account_ids
    }

    pub fn id(&self) -> UserId {
        self.id
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl User {
    /// 新しい空の集約を作成
    pub fn new() -> Self {
        Self::Empty
    }

    /// イベントストリームから集約を再構築
    pub fn from_events(events: Vec<UserEvent>) -> Self {
        let mut user = Self::new();
        for event in events {
            user.apply_event(&event);
        }
        user
    }

    /// コマンドを処理してイベントを生成
    pub fn handle_command(&self, command: UserCommand) -> Result<Vec<UserEvent>, UserError> {
        match command {
            UserCommand::AddAccount {
                user_id,
                account_id,
            } => self.handle_add_account(user_id, account_id),
            UserCommand::CreateUser { user_id } => self.handle_create_user(user_id),
            UserCommand::RemoveAccount {
                user_id,
                account_id,
            } => self.handle_remove_account(user_id, account_id),
        }
    }

    /// イベントを適用して状態を更新
    pub fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::AccountAdded { account_id, .. } => {
                if let User::Active(user) = self {
                    let account_id: AccountId = account_id
                        .parse()
                        .expect("Failed to parse account_id from event");
                    user.account_ids.insert(account_id);
                }
            }
            UserEvent::AccountRemoved { account_id, .. } => {
                if let User::Active(user) = self {
                    let account_id: AccountId = account_id
                        .parse()
                        .expect("Failed to parse account_id from event");
                    user.account_ids.remove(&account_id);
                }
            }
            UserEvent::UserCreated { common } => {
                *self = User::Active(ActiveUser {
                    account_ids: BTreeSet::new(),
                    id: common
                        .user_id
                        .parse()
                        .expect("Failed to parse user_id from event"),
                });
            }
        }
    }

    // コマンドハンドラの実装

    fn handle_add_account(
        &self,
        user_id: UserId,
        account_id: AccountId,
    ) -> Result<Vec<UserEvent>, UserError> {
        match self {
            User::Empty => Err(UserError::UserNotFound),
            User::Active(user) => {
                if user.account_ids.contains(&account_id) {
                    return Err(UserError::AccountAlreadyAdded);
                }

                let common = Self::create_common_props(&user_id);
                Ok(vec![UserEvent::AccountAdded {
                    account_id: account_id.to_string(),
                    common,
                }])
            }
        }
    }

    fn handle_create_user(&self, user_id: UserId) -> Result<Vec<UserEvent>, UserError> {
        if !matches!(self, User::Empty) {
            return Err(UserError::UserAlreadyExists);
        }

        let common = Self::create_common_props(&user_id);
        Ok(vec![UserEvent::UserCreated { common }])
    }

    fn handle_remove_account(
        &self,
        user_id: UserId,
        account_id: AccountId,
    ) -> Result<Vec<UserEvent>, UserError> {
        match self {
            User::Empty => Err(UserError::UserNotFound),
            User::Active(user) => {
                if !user.account_ids.contains(&account_id) {
                    return Err(UserError::AccountNotFound);
                }

                let common = Self::create_common_props(&user_id);
                Ok(vec![UserEvent::AccountRemoved {
                    account_id: account_id.to_string(),
                    common,
                }])
            }
        }
    }

    // ヘルパーメソッド

    fn create_common_props(user_id: &UserId) -> UserEventCommonProps {
        UserEventCommonProps {
            at: date_time::DateTime::now().to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
        }
    }
}

// UserEvent にヘルパーメソッドを追加
impl UserEvent {
    pub fn user_id(&self) -> &String {
        match self {
            UserEvent::AccountAdded { common, .. }
            | UserEvent::AccountRemoved { common, .. }
            | UserEvent::UserCreated { common } => &common.user_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() -> anyhow::Result<()> {
        let user = User::new();
        let user_id = UserId::generate();
        let command = UserCommand::CreateUser { user_id };

        let events = user.handle_command(command)?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            UserEvent::UserCreated { common } => {
                assert_eq!(common.user_id, user_id.to_string());
                Ok(())
            }
            UserEvent::AccountAdded { .. } | UserEvent::AccountRemoved { .. } => {
                anyhow::bail!("Expected UserCreated event")
            }
        }
    }

    #[test]
    fn test_user_from_events() -> anyhow::Result<()> {
        let user_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let common = UserEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-1".to_string(),
            user_id: user_uuid.to_string(),
        };

        let events = vec![UserEvent::UserCreated { common }];

        let user = User::from_events(events);
        match user {
            User::Active(active_user) => {
                assert_eq!(active_user.id().to_string(), user_uuid);
                assert!(active_user.account_ids().is_empty());
                Ok(())
            }
            User::Empty => anyhow::bail!("Expected Active user, got Empty"),
        }
    }

    #[test]
    fn test_user_already_exists() -> anyhow::Result<()> {
        let user_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let common = UserEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-1".to_string(),
            user_id: user_uuid.to_string(),
        };

        let mut user = User::new();
        user.apply_event(&UserEvent::UserCreated { common });

        let new_user_id = UserId::generate();
        let command = UserCommand::CreateUser {
            user_id: new_user_id,
        };

        let result = user.handle_command(command);
        match result {
            Err(UserError::UserAlreadyExists) => Ok(()),
            Err(_) | Ok(_) => anyhow::bail!("Expected UserAlreadyExists error"),
        }
    }

    #[test]
    fn test_add_account() -> anyhow::Result<()> {
        let user_id = UserId::generate();
        let account_id = AccountId::generate();

        // ユーザーを作成
        let mut user = User::new();
        let events = user.handle_command(UserCommand::CreateUser { user_id })?;
        for event in events {
            user.apply_event(&event);
        }

        // アカウントを追加
        let events = user.handle_command(UserCommand::AddAccount { user_id, account_id })?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            UserEvent::AccountAdded {
                account_id: event_account_id,
                common,
            } => {
                assert_eq!(event_account_id, &account_id.to_string());
                assert_eq!(common.user_id, user_id.to_string());
            }
            _ => anyhow::bail!("Expected AccountAdded event"),
        }

        for event in events {
            user.apply_event(&event);
        }

        // アカウントが追加されていることを確認
        match &user {
            User::Active(active_user) => {
                assert!(active_user.account_ids().contains(&account_id));
                Ok(())
            }
            User::Empty => anyhow::bail!("Expected Active user"),
        }
    }

    #[test]
    fn test_add_account_already_added() -> anyhow::Result<()> {
        let user_id = UserId::generate();
        let account_id = AccountId::generate();

        let mut user = User::new();
        let events = user.handle_command(UserCommand::CreateUser { user_id })?;
        for event in events {
            user.apply_event(&event);
        }

        let events = user.handle_command(UserCommand::AddAccount { user_id, account_id })?;
        for event in events {
            user.apply_event(&event);
        }

        // 同じアカウントを再度追加しようとする
        let result = user.handle_command(UserCommand::AddAccount { user_id, account_id });
        match result {
            Err(UserError::AccountAlreadyAdded) => Ok(()),
            Err(_) | Ok(_) => anyhow::bail!("Expected AccountAlreadyAdded error"),
        }
    }

    #[test]
    fn test_remove_account() -> anyhow::Result<()> {
        let user_id = UserId::generate();
        let account_id = AccountId::generate();

        // ユーザーを作成
        let mut user = User::new();
        let events = user.handle_command(UserCommand::CreateUser { user_id })?;
        for event in events {
            user.apply_event(&event);
        }

        // アカウントを追加
        let events = user.handle_command(UserCommand::AddAccount { user_id, account_id })?;
        for event in events {
            user.apply_event(&event);
        }

        // アカウントを削除
        let events = user.handle_command(UserCommand::RemoveAccount { user_id, account_id })?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            UserEvent::AccountRemoved {
                account_id: event_account_id,
                common,
            } => {
                assert_eq!(event_account_id, &account_id.to_string());
                assert_eq!(common.user_id, user_id.to_string());
            }
            _ => anyhow::bail!("Expected AccountRemoved event"),
        }

        for event in events {
            user.apply_event(&event);
        }

        // アカウントが削除されていることを確認
        match &user {
            User::Active(active_user) => {
                assert!(!active_user.account_ids().contains(&account_id));
                Ok(())
            }
            User::Empty => anyhow::bail!("Expected Active user"),
        }
    }

    #[test]
    fn test_remove_account_not_found() -> anyhow::Result<()> {
        let user_id = UserId::generate();
        let account_id = AccountId::generate();

        let mut user = User::new();
        let events = user.handle_command(UserCommand::CreateUser { user_id })?;
        for event in events {
            user.apply_event(&event);
        }

        // 存在しないアカウントを削除しようとする
        let result = user.handle_command(UserCommand::RemoveAccount { user_id, account_id });
        match result {
            Err(UserError::AccountNotFound) => Ok(()),
            Err(_) | Ok(_) => anyhow::bail!("Expected AccountNotFound error"),
        }
    }

    #[test]
    fn test_add_account_user_not_found() -> anyhow::Result<()> {
        let user = User::new();
        let user_id = UserId::generate();
        let account_id = AccountId::generate();

        let result = user.handle_command(UserCommand::AddAccount { user_id, account_id });
        match result {
            Err(UserError::UserNotFound) => Ok(()),
            Err(_) | Ok(_) => anyhow::bail!("Expected UserNotFound error"),
        }
    }
}
