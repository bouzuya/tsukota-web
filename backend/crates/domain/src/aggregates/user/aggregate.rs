use crate::UserCommand;
use crate::UserEvent;
use crate::UserEventCommonProps;
use crate::UserId;

/// ユーザー集約のエラー
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User already exists")]
    UserAlreadyExists,
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
    /// ユーザー ID
    id: UserId,
}

impl ActiveUser {
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
            UserCommand::CreateUser { user_id } => self.handle_create_user(user_id),
        }
    }

    /// イベントを適用して状態を更新
    pub fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::UserCreated { common } => {
                *self = User::Active(ActiveUser {
                    id: common
                        .user_id
                        .parse()
                        .expect("Failed to parse user_id from event"),
                });
            }
        }
    }

    // コマンドハンドラの実装

    fn handle_create_user(&self, user_id: UserId) -> Result<Vec<UserEvent>, UserError> {
        if !matches!(self, User::Empty) {
            return Err(UserError::UserAlreadyExists);
        }

        let common = Self::create_common_props(&user_id);
        Ok(vec![UserEvent::UserCreated { common }])
    }

    // ヘルパーメソッド

    fn create_common_props(user_id: &UserId) -> UserEventCommonProps {
        UserEventCommonProps {
            at: date_time::DateTime::now().into(),
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
        }
    }
}

// UserEvent にヘルパーメソッドを追加
impl UserEvent {
    pub fn user_id(&self) -> &String {
        match self {
            UserEvent::UserCreated { common } => &common.user_id,
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
            User::Active(ActiveUser { id }) => {
                assert_eq!(id.to_string(), user_uuid);
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
            Ok(_) => anyhow::bail!("Expected UserAlreadyExists error, but command succeeded"),
        }
    }
}
