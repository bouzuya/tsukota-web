use crate::UserId;

/// ユーザー集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum UserCommand {
    /// ユーザーを作成する
    CreateUser { user_id: UserId },
}
