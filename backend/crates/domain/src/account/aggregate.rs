use super::commands::AccountCommand;
use super::events::{AccountEvent, AccountId, CategoryId, TransactionId, UserId};
use std::collections::{HashMap, HashSet};
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
pub struct Account {
    /// アカウント ID
    pub id: Option<AccountId>,
    /// アカウント名
    pub name: Option<String>,
    /// オーナー ID
    pub owner_id: Option<UserId>,
    /// メンバー ID のセット
    pub member_ids: HashSet<UserId>,
    /// カテゴリのマップ
    pub categories: HashMap<CategoryId, Category>,
    /// 取引のマップ
    pub transactions: HashMap<TransactionId, Transaction>,
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

impl Account {
    /// 新しい空の集約を作成
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            owner_id: None,
            member_ids: HashSet::new(),
            categories: HashMap::new(),
            transactions: HashMap::new(),
        }
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
    pub fn handle_command(&self, command: AccountCommand) -> Result<Vec<AccountEvent>, AccountError> {
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
            } => self.handle_update_transaction(
                transaction_id,
                amount,
                category_id,
                date,
                memo,
            ),

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
                self.id = Some(account_id.clone());
                self.name = Some(name.clone());
                self.owner_id = Some(owner_id.clone());
                self.member_ids.insert(owner_id.clone());
            }

            AccountEvent::AccountRenamed { name } => {
                self.name = Some(name.clone());
            }

            AccountEvent::MemberAdded { user_id } => {
                self.member_ids.insert(user_id.clone());
            }

            AccountEvent::MemberRemoved { user_id } => {
                self.member_ids.remove(user_id);
            }

            AccountEvent::CategoryCreated {
                category_id,
                name,
                order,
            } => {
                self.categories.insert(
                    category_id.clone(),
                    Category {
                        id: category_id.clone(),
                        name: name.clone(),
                        order: *order,
                        deleted: false,
                    },
                );
            }

            AccountEvent::CategoryRenamed { category_id, name } => {
                if let Some(category) = self.categories.get_mut(category_id) {
                    category.name = name.clone();
                }
            }

            AccountEvent::CategoryReordered { category_id, order } => {
                if let Some(category) = self.categories.get_mut(category_id) {
                    category.order = *order;
                }
            }

            AccountEvent::CategoryDeleted { category_id } => {
                if let Some(category) = self.categories.get_mut(category_id) {
                    category.deleted = true;
                }
            }

            AccountEvent::TransactionCreated {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
                created_by,
            } => {
                self.transactions.insert(
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

            AccountEvent::TransactionUpdated {
                transaction_id,
                amount,
                category_id,
                date,
                memo,
            } => {
                if let Some(transaction) = self.transactions.get_mut(transaction_id) {
                    transaction.amount = *amount;
                    transaction.category_id = category_id.clone();
                    transaction.date = date.clone();
                    transaction.memo = memo.clone();
                }
            }

            AccountEvent::TransactionDeleted { transaction_id } => {
                self.transactions.remove(transaction_id);
            }
        }
    }

    // コマンドハンドラの実装

    fn handle_create_account(
        &self,
        account_id: AccountId,
        name: String,
        owner_id: UserId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        if self.id.is_some() {
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyAccountName);
        }

        Ok(vec![AccountEvent::AccountRenamed { name }])
    }

    fn handle_add_member(&self, user_id: UserId) -> Result<Vec<AccountEvent>, AccountError> {
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if self.member_ids.contains(&user_id) {
            return Err(AccountError::MemberAlreadyExists);
        }

        Ok(vec![AccountEvent::MemberAdded { user_id }])
    }

    fn handle_remove_member(&self, user_id: UserId) -> Result<Vec<AccountEvent>, AccountError> {
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if Some(&user_id) == self.owner_id.as_ref() {
            return Err(AccountError::CannotRemoveOwner);
        }

        if !self.member_ids.contains(&user_id) {
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
        if self.id.is_none() {
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        let category = self
            .categories
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        let category = self
            .categories
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        let category = self
            .categories
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if amount == 0 {
            return Err(AccountError::InvalidAmount);
        }

        // カテゴリの存在確認（削除されていても既存の取引では使用可能）
        self.categories
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if !self.transactions.contains_key(&transaction_id) {
            return Err(AccountError::TransactionNotFound);
        }

        if amount == 0 {
            return Err(AccountError::InvalidAmount);
        }

        // カテゴリの存在確認
        self.categories
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
        if self.id.is_none() {
            return Err(AccountError::AccountNotFound);
        }

        if !self.transactions.contains_key(&transaction_id) {
            return Err(AccountError::TransactionNotFound);
        }

        Ok(vec![AccountEvent::TransactionDeleted { transaction_id }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() {
        let account = Account::new();
        let command = AccountCommand::CreateAccount {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        };

        let events = account.handle_command(command).unwrap();
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
            }
            _ => panic!("Expected AccountCreated event"),
        }
    }

    #[test]
    fn test_account_from_events() {
        let events = vec![AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        }];

        let account = Account::from_events(events);
        assert_eq!(account.id, Some("acc-1".to_string()));
        assert_eq!(account.name, Some("My Account".to_string()));
        assert_eq!(account.owner_id, Some("user-1".to_string()));
        assert!(account.member_ids.contains("user-1"));
    }

    #[test]
    fn test_add_member() {
        let mut account = Account::new();
        account.apply_event(&AccountEvent::AccountCreated {
            account_id: "acc-1".to_string(),
            name: "My Account".to_string(),
            owner_id: "user-1".to_string(),
        });

        let command = AccountCommand::AddMember {
            user_id: "user-2".to_string(),
        };

        let events = account.handle_command(command).unwrap();
        assert_eq!(events.len(), 1);

        account.apply_event(&events[0]);
        assert!(account.member_ids.contains("user-2"));
    }

    #[test]
    fn test_cannot_remove_owner() {
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
        assert!(matches!(result, Err(AccountError::CannotRemoveOwner)));
    }
}
