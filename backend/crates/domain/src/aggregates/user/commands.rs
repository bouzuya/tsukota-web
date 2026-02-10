use crate::AccountId;
use crate::UserId;

/// ユーザー集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum UserCommand {
    /// アカウントを追加する
    AddAccount {
        user_id: UserId,
        account_id: AccountId,
    },
    /// ユーザーを作成する
    CreateUser { user_id: UserId },
    /// アカウントを削除する
    RemoveAccount {
        user_id: UserId,
        account_id: AccountId,
    },
}
