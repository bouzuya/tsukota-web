use crate::value_objects::AccountId;
use crate::value_objects::CategoryId;
use crate::value_objects::TransactionId;
use crate::value_objects::UserId;

/// アカウント集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum AccountCommand {
    /// 区分を追加する
    AddCategory {
        category_id: CategoryId,
        name: String,
    },

    /// オーナーを追加する
    AddOwner { owner: UserId },

    /// 取引を追加する
    AddTransaction {
        transaction_id: TransactionId,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    },

    /// アカウントを作成する
    CreateAccount {
        account_id: AccountId,
        name: String,
        owners: Vec<UserId>,
    },

    /// アカウントを削除する
    DeleteAccount,

    /// 区分を削除する（論理削除）
    DeleteCategory { category_id: CategoryId },

    /// 取引を削除する
    DeleteTransaction { transaction_id: TransactionId },

    /// オーナーを削除する
    RemoveOwner { owner: UserId },

    /// アカウント名を変更する
    UpdateAccount { name: String },

    /// 区分名を変更する
    UpdateCategory {
        category_id: CategoryId,
        name: String,
    },

    /// 取引を更新する
    UpdateTransaction {
        transaction_id: TransactionId,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    },
}
