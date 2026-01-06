use super::commands::AccountCommand;
use super::events::{AccountEvent, AccountId, CategoryId, TransactionId, UserId};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// アカウント集約のエラー
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account already exists")]
    AccountAlreadyExists,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Account name cannot be empty")]
    EmptyAccountName,

    #[error("Member already exists")]
    MemberAlreadyExists,

    #[error("Member not found")]
    MemberNotFound,

    #[error("Cannot remove owner")]
    CannotRemoveOwner,

    #[error("Category not found")]
    CategoryNotFound,

    #[error("Category already deleted")]
    CategoryAlreadyDeleted,

    #[error("Category name cannot be empty")]
    EmptyCategoryName,

    #[error("Transaction not found")]
    TransactionNotFound,

    #[error("Invalid date format")]
    InvalidDateFormat,

    #[error("Amount must be greater than 0")]
    InvalidAmount,
}

/// カテゴリの状態
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub order: u32,
    pub deleted: bool,
}

/// 取引の状態
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub id: TransactionId,
    pub amount: u64,
    pub category_id: CategoryId,
    pub date: String,
    pub memo: Option<String>,
    pub created_by: UserId,
}

/// アカウント集約
#[derive(Debug, Clone, PartialEq)]
pub enum Account {
    /// アカウント作成前の空の状態
    Empty,
    /// アカウント作成後のアクティブな状態
    Active {
        /// アカウント ID
        id: AccountId,
        /// アカウント名
        name: String,
        /// オーナー ID
        owner_id: UserId,
        /// メンバー ID のセット (オーナーを含む)
        member_ids: BTreeSet<UserId>,
        /// カテゴリのマップ
        categories: BTreeMap<CategoryId, Category>,
        /// 取引のマップ
        transactions: BTreeMap<TransactionId, Transaction>,
    },
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

impl Account {
    /// 新しい空の集約を作成
    pub fn new() -> Self {
        Self::Empty
    }

    /// イベントストリームから集約を再構築
    pub fn from_events(events: Vec<AccountEvent>) -> Self {
        let mut account = Self::new();
        for event in events {
            account.apply_event(&event);
        }
        account
    }

    /// コマンドを処理してイベントを生成
    pub fn handle_command(
        &self,
        command: AccountCommand,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        match command {
            AccountCommand::CreateAccount {
                account_id,
                name,
                owner_id,
            } => self.handle_create_account(account_id, name, owner_id),

            AccountCommand::RenameAccount { name } => self.handle_rename_account(name),

            AccountCommand::AddMember { user_id } => self.handle_add_member(user_id),

            AccountCommand::RemoveMember { user_id } => self.handle_remove_member(user_id),

            AccountCommand::CreateCategory {
                category_id,
                name,
                order,
            } => self.handle_create_category(category_id, name, order),

            AccountCommand::RenameCategory { category_id, name } => {
                self.handle_rename_category(category_id, name)
            }

            AccountCommand::ReorderCategory { category_id, order } => {
                self.handle_reorder_category(category_id, order)
            }

            AccountCommand::DeleteCategory { category_id } => {
                self.handle_delete_category(category_id)
            }

            AccountCommand::CreateTransaction {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
                created_by,
            } => self.handle_create_transaction(
                transaction_id,
                amount,
                category_id,
                date,
                memo,
                created_by,
            ),

            AccountCommand::UpdateTransaction {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
            } => self.handle_update_transaction(transaction_id, amount, category_id, date, memo),

            AccountCommand::DeleteTransaction { transaction_id } => {
                self.handle_delete_transaction(transaction_id)
            }
        }
    }

    /// イベントを適用して状態を更新
    pub fn apply_event(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::AccountCreated {
                account_id,
                name,
                owner_id,
            } => {
                let mut member_ids = BTreeSet::new();
                member_ids.insert(owner_id.clone());
                *self = Account::Active {
                    id: account_id.clone(),
                    name: name.clone(),
                    owner_id: owner_id.clone(),
                    member_ids,
                    categories: BTreeMap::new(),
                    transactions: BTreeMap::new(),
                };
            }

            AccountEvent::AccountRenamed { name } => match self {
                Account::Active {
                    name: current_name, ..
                } => {
                    *current_name = name.clone();
                }
                Account::Empty => unreachable!("AccountRenamed event applied to Empty account"),
            },

            AccountEvent::MemberAdded { user_id } => match self {
                Account::Active { member_ids, .. } => {
                    member_ids.insert(user_id.clone());
                }
                Account::Empty => unreachable!("MemberAdded event applied to Empty account"),
            },

            AccountEvent::MemberRemoved { user_id } => match self {
                Account::Active { member_ids, .. } => {
                    member_ids.remove(user_id);
                }
                Account::Empty => unreachable!("MemberRemoved event applied to Empty account"),
            },

            AccountEvent::CategoryCreated {
                category_id,
                name,
                order,
            } => match self {
                Account::Active { categories, .. } => {
                    categories.insert(
                        category_id.clone(),
                        Category {
                            id: category_id.clone(),
                            name: name.clone(),
                            order: *order,
                            deleted: false,
                        },
                    );
                }
                Account::Empty => {
                    unreachable!("CategoryCreated event applied to Empty account")
                }
            },

            AccountEvent::CategoryRenamed { category_id, name } => match self {
                Account::Active { categories, .. } => {
                    if let Some(category) = categories.get_mut(category_id) {
                        category.name = name.clone();
                    }
                }
                Account::Empty => {
                    unreachable!("CategoryRenamed event applied to Empty account")
                }
            },

            AccountEvent::CategoryReordered { category_id, order } => match self {
                Account::Active { categories, .. } => {
                    if let Some(category) = categories.get_mut(category_id) {
                        category.order = *order;
                    }
                }
                Account::Empty => {
                    unreachable!("CategoryReordered event applied to Empty account")
                }
            },

            AccountEvent::CategoryDeleted { category_id } => match self {
                Account::Active { categories, .. } => {
                    if let Some(category) = categories.get_mut(category_id) {
                        category.deleted = true;
                    }
                }
                Account::Empty => {
                    unreachable!("CategoryDeleted event applied to Empty account")
                }
            },

            AccountEvent::TransactionCreated {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
                created_by,
            } => match self {
                Account::Active { transactions, .. } => {
                    transactions.insert(
                        transaction_id.clone(),
                        Transaction {
                            id: transaction_id.clone(),
                            amount: *amount,
                            category_id: category_id.clone(),
                            date: date.clone(),
                            memo: memo.clone(),
                            created_by: created_by.clone(),
                        },
                    );
                }
                Account::Empty => {
                    unreachable!("TransactionCreated event applied to Empty account")
                }
            },

            AccountEvent::TransactionUpdated {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
            } => match self {
                Account::Active { transactions, .. } => {
                    if let Some(transaction) = transactions.get_mut(transaction_id) {
                        transaction.amount = *amount;
                        transaction.category_id = category_id.clone();
                        transaction.date = date.clone();
                        transaction.memo = memo.clone();
                    }
                }
                Account::Empty => {
                    unreachable!("TransactionUpdated event applied to Empty account")
                }
            },

            AccountEvent::TransactionDeleted { transaction_id } => match self {
                Account::Active { transactions, .. } => {
                    transactions.remove(transaction_id);
                }
                Account::Empty => {
                    unreachable!("TransactionDeleted event applied to Empty account")
                }
            },
        }
    }

    // コマンドハンドラの実装

    fn handle_create_account(
        &self,
        account_id: AccountId,
        name: String,
        owner_id: UserId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        if !matches!(self, Account::Empty) {
            return Err(AccountError::AccountAlreadyExists);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyAccountName);
        }

        Ok(vec![AccountEvent::AccountCreated {
            account_id,
            name,
            owner_id,
        }])
    }

    fn handle_rename_account(&self, name: String) -> Result<Vec<AccountEvent>, AccountError> {
        if !matches!(self, Account::Active { .. }) {
            return Err(AccountError::AccountNotFound);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyAccountName);
        }

        Ok(vec![AccountEvent::AccountRenamed { name }])
    }

    fn handle_add_member(&self, user_id: UserId) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { member_ids, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if member_ids.contains(&user_id) {
            return Err(AccountError::MemberAlreadyExists);
        }

        Ok(vec![AccountEvent::MemberAdded { user_id }])
    }

    fn handle_remove_member(&self, user_id: UserId) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active {
            owner_id,
            member_ids,
            ..
        } = self
        else {
            return Err(AccountError::AccountNotFound);
        };

        if &user_id == owner_id {
            return Err(AccountError::CannotRemoveOwner);
        }

        if !member_ids.contains(&user_id) {
            return Err(AccountError::MemberNotFound);
        }

        Ok(vec![AccountEvent::MemberRemoved { user_id }])
    }

    fn handle_create_category(
        &self,
        category_id: CategoryId,
        name: String,
        order: u32,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        if !matches!(self, Account::Active { .. }) {
            return Err(AccountError::AccountNotFound);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyCategoryName);
        }

        Ok(vec![AccountEvent::CategoryCreated {
            category_id,
            name,
            order,
        }])
    }

    fn handle_rename_category(
        &self,
        category_id: CategoryId,
        name: String,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        let category = categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        if category.deleted {
            return Err(AccountError::CategoryAlreadyDeleted);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyCategoryName);
        }

        Ok(vec![AccountEvent::CategoryRenamed { category_id, name }])
    }

    fn handle_reorder_category(
        &self,
        category_id: CategoryId,
        order: u32,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        let category = categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        if category.deleted {
            return Err(AccountError::CategoryAlreadyDeleted);
        }

        Ok(vec![AccountEvent::CategoryReordered { category_id, order }])
    }

    fn handle_delete_category(
        &self,
        category_id: CategoryId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        let category = categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        if category.deleted {
            return Err(AccountError::CategoryAlreadyDeleted);
        }

        Ok(vec![AccountEvent::CategoryDeleted { category_id }])
    }

    fn handle_create_transaction(
        &self,
        transaction_id: TransactionId,
        amount: u64,
        category_id: CategoryId,
        date: String,
        memo: Option<String>,
        created_by: UserId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if amount == 0 {
            return Err(AccountError::InvalidAmount);
        }

        // カテゴリの存在確認（削除されていても既存の取引では使用可能）
        categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        // 簡易的な日付形式チェック
        if !date.contains('-') {
            return Err(AccountError::InvalidDateFormat);
        }

        Ok(vec![AccountEvent::TransactionCreated {
            transaction_id,
            amount,
            category_id,
            date,
            memo,
            created_by,
        }])
    }

    fn handle_update_transaction(
        &self,
        transaction_id: TransactionId,
        amount: u64,
        category_id: CategoryId,
        date: String,
        memo: Option<String>,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active {
            categories,
            transactions,
            ..
        } = self
        else {
            return Err(AccountError::AccountNotFound);
        };

        if !transactions.contains_key(&transaction_id) {
            return Err(AccountError::TransactionNotFound);
        }

        if amount == 0 {
            return Err(AccountError::InvalidAmount);
        }

        // カテゴリの存在確認
        categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        // 簡易的な日付形式チェック
        if !date.contains('-') {
            return Err(AccountError::InvalidDateFormat);
        }

        Ok(vec![AccountEvent::TransactionUpdated {
            transaction_id,
            amount,
            category_id,
            date,
            memo,
        }])
    }

    fn handle_delete_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { transactions, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if !transactions.contains_key(&transaction_id) {
            return Err(AccountError::TransactionNotFound);
        }

        Ok(vec![AccountEvent::TransactionDeleted { transaction_id }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() -> anyhow::Result<()> {
        let account = Account::new();
        let command = AccountCommand::CreateAccount {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        };

        let events = account.handle_command(command)?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            AccountEvent::AccountCreated {
                account_id,
                name,
                owner_id,
            } => {
                assert_eq!(account_id, "acc-1");
                assert_eq!(name, "My Account");
                assert_eq!(owner_id, "user-1");
                Ok(())
            }
            event => anyhow::bail!("Expected AccountCreated event, got {:?}", event),
        }
    }

    #[test]
    fn test_account_from_events() -> anyhow::Result<()> {
        let events = vec![AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        }];

        let account = Account::from_events(events);
        match account {
            Account::Active {
                id,
                name,
                owner_id,
                member_ids,
                ..
            } => {
                assert_eq!(id, "acc-1");
                assert_eq!(name, "My Account");
                assert_eq!(owner_id, "user-1");
                assert!(member_ids.contains("user-1"));
                Ok(())
            }
            Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
        }
    }

    #[test]
    fn test_add_member() -> anyhow::Result<()> {
        let mut account = Account::new();
        account.apply_event(&AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        });

        let command = AccountCommand::AddMember {
            user_id: "user-2".to_string(),
        };

        let events = account.handle_command(command)?;
        assert_eq!(events.len(), 1);

        account.apply_event(&events[0]);
        match account {
            Account::Active { member_ids, .. } => {
                assert!(member_ids.contains("user-2"));
                Ok(())
            }
            Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
        }
    }

    #[test]
    fn test_cannot_remove_owner() -> anyhow::Result<()> {
        let mut account = Account::new();
        account.apply_event(&AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        });

        let command = AccountCommand::RemoveMember {
            user_id: "user-1".to_string(),
        };

        let result = account.handle_command(command);
        match result {
            Err(AccountError::CannotRemoveOwner) => Ok(()),
            Ok(_) => anyhow::bail!("Expected CannotRemoveOwner error, but command succeeded"),
            Err(e) => anyhow::bail!("Expected CannotRemoveOwner error, got {:?}", e),
        }
    }
}
