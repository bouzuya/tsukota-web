use super::value_objects::{AccountId, CategoryId};

/// アカウント集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum AccountCommand {
    /// アカウントを作成する
    CreateAccount {
        account_id: AccountId,
        name: String,
        owners: Vec<String>,
    },

    /// アカウントを削除する
    DeleteAccount,

    /// アカウント名を変更する
    UpdateAccount { name: String },

    /// オーナーを追加する
    AddOwner { owner: String },

    /// オーナーを削除する
    RemoveOwner { owner: String },

    /// カテゴリを追加する
    AddCategory { category_id: CategoryId, name: String },

    /// カテゴリ名を変更する
    UpdateCategory {
        category_id: CategoryId,
        name: String,
    },

    /// カテゴリを削除する（論理削除）
    DeleteCategory { category_id: CategoryId },

    /// 取引を追加する
    AddTransaction {
        transaction_id: String,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    },

    /// 取引を更新する
    UpdateTransaction {
        transaction_id: String,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    },

    /// 取引を削除する
    DeleteTransaction { transaction_id: String },
}
