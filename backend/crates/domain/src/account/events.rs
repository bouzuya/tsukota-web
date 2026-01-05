use serde::{Deserialize, Serialize};

/// アカウント ID
pub type AccountId = String;

/// ユーザー ID
pub type UserId = String;

/// カテゴリ ID
pub type CategoryId = String;

/// 取引 ID
pub type TransactionId = String;

/// アカウント集約に対するイベント
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AccountEvent {
    /// アカウントが作成された
    AccountCreated {
        account_id: AccountId,
        name: String,
        owner_id: UserId,
    },

    /// アカウント名が変更された
    AccountRenamed {
        name: String,
    },

    /// メンバーが追加された
    MemberAdded {
        user_id: UserId,
    },

    /// メンバーが削除された
    MemberRemoved {
        user_id: UserId,
    },

    /// カテゴリが作成された
    CategoryCreated {
        category_id: CategoryId,
        name: String,
        category_type: CategoryType,
        order: u32,
    },

    /// カテゴリ名が変更された
    CategoryRenamed {
        category_id: CategoryId,
        name: String,
    },

    /// カテゴリの表示順が変更された
    CategoryReordered {
        category_id: CategoryId,
        order: u32,
    },

    /// カテゴリが削除された（論理削除）
    CategoryDeleted {
        category_id: CategoryId,
    },

    /// 取引が作成された
    TransactionCreated {
        transaction_id: TransactionId,
        transaction_type: TransactionType,
        amount: u64,
        category_id: CategoryId,
        date: String, // ISO 8601 format (YYYY-MM-DD)
        memo: Option<String>,
        created_by: UserId,
    },

    /// 取引が更新された
    TransactionUpdated {
        transaction_id: TransactionId,
        transaction_type: TransactionType,
        amount: u64,
        category_id: CategoryId,
        date: String,
        memo: Option<String>,
    },

    /// 取引が削除された
    TransactionDeleted {
        transaction_id: TransactionId,
    },
}

/// カテゴリの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Income,
    Expense,
}

/// 取引の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Income,
    Expense,
}
