use super::events::{AccountId, CategoryId, TransactionId, UserId};

/// アカウント集約に対するコマンド
#[derive(Debug, Clone, PartialEq)]
pub enum AccountCommand {
    /// アカウントを作成する
    CreateAccount {
        account_id: AccountId,
        name: String,
        owner_id: UserId,
    },

    /// アカウント名を変更する
    RenameAccount {
        name: String,
    },

    /// メンバーを追加する
    AddMember {
        user_id: UserId,
    },

    /// メンバーを削除する
    RemoveMember {
        user_id: UserId,
    },

    /// カテゴリを作成する
    CreateCategory {
        category_id: CategoryId,
        name: String,
        order: u32,
    },

    /// カテゴリ名を変更する
    RenameCategory {
        category_id: CategoryId,
        name: String,
    },

    /// カテゴリの表示順を変更する
    ReorderCategory {
        category_id: CategoryId,
        order: u32,
    },

    /// カテゴリを削除する（論理削除）
    DeleteCategory {
        category_id: CategoryId,
    },

    /// 取引を作成する
    CreateTransaction {
        transaction_id: TransactionId,
        amount: u64,
        category_id: CategoryId,
        date: String, // ISO 8601 format (YYYY-MM-DD)
        memo: Option<String>,
        created_by: UserId,
    },

    /// 取引を更新する
    UpdateTransaction {
        transaction_id: TransactionId,
        amount: u64,
        category_id: CategoryId,
        date: String,
        memo: Option<String>,
    },

    /// 取引を削除する
    DeleteTransaction {
        transaction_id: TransactionId,
    },
}
