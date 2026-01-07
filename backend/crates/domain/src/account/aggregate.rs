use super::commands::AccountCommand;
use super::events::AccountEvent;
use super::events::AccountEventCommonProps;
use super::events::TransactionProps;
use super::value_objects::AccountId;
use super::value_objects::CategoryId;
use super::value_objects::TransactionId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// アカウント集約のエラー
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("Account already exists")]
    AccountAlreadyExists,

    #[error("Account not found")]
    AccountNotFound,

    #[error("Account name cannot be empty")]
    EmptyAccountName,

    #[error("Owner already exists")]
    OwnerAlreadyExists,

    #[error("Owner not found")]
    OwnerNotFound,

    #[error("Cannot remove the last owner")]
    CannotRemoveLastOwner,

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

    #[error("Amount cannot be empty")]
    EmptyAmount,
}

/// カテゴリの状態
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub deleted: bool,
}

/// 取引の状態
#[derive(Clone, Debug, PartialEq)]
pub struct Transaction {
    pub id: TransactionId,
    pub amount: String,
    pub category_id: CategoryId,
    pub comment: String,
    pub date: String,
}

/// アカウント集約
#[derive(Clone, Debug, PartialEq)]
pub enum Account {
    /// アカウント作成前の空の状態
    Empty,
    /// アカウント作成後のアクティブな状態
    Active {
        /// アカウント ID
        id: AccountId,
        /// アカウント名
        name: String,
        /// オーナーのセット
        owners: BTreeSet<String>,
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
                owners,
            } => self.handle_create_account(account_id, name, owners),

            AccountCommand::DeleteAccount => self.handle_delete_account(),

            AccountCommand::UpdateAccount { name } => self.handle_update_account(name),

            AccountCommand::AddOwner { owner } => self.handle_add_owner(owner),

            AccountCommand::RemoveOwner { owner } => self.handle_remove_owner(owner),

            AccountCommand::AddCategory { category_id, name } => {
                self.handle_add_category(category_id, name)
            }

            AccountCommand::UpdateCategory { category_id, name } => {
                self.handle_update_category(category_id, name)
            }

            AccountCommand::DeleteCategory { category_id } => {
                self.handle_delete_category(category_id)
            }

            AccountCommand::AddTransaction {
                transaction_id,
                amount,
                category_id,
                comment,
                date,
            } => self.handle_add_transaction(transaction_id, amount, category_id, comment, date),

            AccountCommand::UpdateTransaction {
                transaction_id,
                amount,
                category_id,
                comment,
                date,
            } => self.handle_update_transaction(transaction_id, amount, category_id, comment, date),

            AccountCommand::DeleteTransaction { transaction_id } => {
                self.handle_delete_transaction(transaction_id)
            }
        }
    }

    /// イベントを適用して状態を更新
    pub fn apply_event(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::AccountCreated { name, owners, .. } => {
                let owners_set: BTreeSet<String> = owners.iter().cloned().collect();
                *self = Account::Active {
                    id: event
                        .account_id()
                        .parse()
                        .expect("Failed to parse account_id from event"),
                    name: name.clone(),
                    owners: owners_set,
                    categories: BTreeMap::new(),
                    transactions: BTreeMap::new(),
                };
            }

            AccountEvent::AccountDeleted { .. } => {
                *self = Account::Empty;
            }

            AccountEvent::AccountUpdated { name, .. } => match self {
                Account::Active {
                    name: current_name, ..
                } => {
                    *current_name = name.clone();
                }
                Account::Empty => unreachable!("AccountUpdated event applied to Empty account"),
            },

            AccountEvent::OwnerAdded { owner, .. } => match self {
                Account::Active { owners, .. } => {
                    owners.insert(owner.clone());
                }
                Account::Empty => unreachable!("OwnerAdded event applied to Empty account"),
            },

            AccountEvent::OwnerRemoved { owner, .. } => match self {
                Account::Active { owners, .. } => {
                    owners.remove(owner);
                }
                Account::Empty => unreachable!("OwnerRemoved event applied to Empty account"),
            },

            AccountEvent::CategoryAdded {
                category_id, name, ..
            } => match self {
                Account::Active { categories, .. } => {
                    let id: CategoryId = category_id
                        .parse()
                        .expect("Failed to parse category_id from event");
                    categories.insert(
                        id,
                        Category {
                            id,
                            name: name.clone(),
                            deleted: false,
                        },
                    );
                }
                Account::Empty => {
                    unreachable!("CategoryAdded event applied to Empty account")
                }
            },

            AccountEvent::CategoryUpdated {
                category_id, name, ..
            } => match self {
                Account::Active { categories, .. } => {
                    let id: CategoryId = category_id
                        .parse()
                        .expect("Failed to parse category_id from event");
                    if let Some(category) = categories.get_mut(&id) {
                        category.name = name.clone();
                    }
                }
                Account::Empty => {
                    unreachable!("CategoryUpdated event applied to Empty account")
                }
            },

            AccountEvent::CategoryDeleted { category_id, .. } => match self {
                Account::Active { categories, .. } => {
                    let id: CategoryId = category_id
                        .parse()
                        .expect("Failed to parse category_id from event");
                    if let Some(category) = categories.get_mut(&id) {
                        category.deleted = true;
                    }
                }
                Account::Empty => {
                    unreachable!("CategoryDeleted event applied to Empty account")
                }
            },

            AccountEvent::TransactionAdded {
                transaction_id,
                props,
                ..
            } => match self {
                Account::Active { transactions, .. } => {
                    let id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id from event");
                    transactions.insert(
                        id,
                        Transaction {
                            id,
                            amount: props.amount.clone(),
                            category_id: props
                                .category_id
                                .parse()
                                .expect("Failed to parse category_id from event"),
                            comment: props.comment.clone(),
                            date: props.date.clone(),
                        },
                    );
                }
                Account::Empty => {
                    unreachable!("TransactionAdded event applied to Empty account")
                }
            },

            AccountEvent::TransactionUpdated {
                transaction_id,
                props,
                ..
            } => match self {
                Account::Active { transactions, .. } => {
                    let id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id from event");
                    if let Some(transaction) = transactions.get_mut(&id) {
                        transaction.amount = props.amount.clone();
                        transaction.category_id = props
                            .category_id
                            .parse()
                            .expect("Failed to parse category_id from event");
                        transaction.comment = props.comment.clone();
                        transaction.date = props.date.clone();
                    }
                }
                Account::Empty => {
                    unreachable!("TransactionUpdated event applied to Empty account")
                }
            },

            AccountEvent::TransactionDeleted { transaction_id, .. } => match self {
                Account::Active { transactions, .. } => {
                    let id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id from event");
                    transactions.remove(&id);
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
        owners: Vec<String>,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        if !matches!(self, Account::Empty) {
            return Err(AccountError::AccountAlreadyExists);
        }

        if name.trim().is_empty() {
            return Err(AccountError::EmptyAccountName);
        }

        let common = Self::create_common_props(&account_id);
        Ok(vec![AccountEvent::AccountCreated {
            name,
            owners,
            common,
        }])
    }

    fn handle_delete_account(&self) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::AccountDeleted { common }])
    }

    fn handle_update_account(&self, name: String) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if name.trim().is_empty() {
            return Err(AccountError::EmptyAccountName);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::AccountUpdated { name, common }])
    }

    fn handle_add_owner(&self, owner: String) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, owners, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if owners.contains(&owner) {
            return Err(AccountError::OwnerAlreadyExists);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::OwnerAdded { owner, common }])
    }

    fn handle_remove_owner(&self, owner: String) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, owners, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if !owners.contains(&owner) {
            return Err(AccountError::OwnerNotFound);
        }

        if owners.len() == 1 {
            return Err(AccountError::CannotRemoveLastOwner);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::OwnerRemoved { owner, common }])
    }

    fn handle_add_category(
        &self,
        category_id: CategoryId,
        name: String,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if name.trim().is_empty() {
            return Err(AccountError::EmptyCategoryName);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::CategoryAdded {
            category_id: category_id.to_string(),
            name,
            common,
        }])
    }

    fn handle_update_category(
        &self,
        category_id: CategoryId,
        name: String,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, categories, .. } = self else {
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

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::CategoryUpdated {
            category_id: category_id.to_string(),
            name,
            common,
        }])
    }

    fn handle_delete_category(
        &self,
        category_id: CategoryId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        let category = categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        if category.deleted {
            return Err(AccountError::CategoryAlreadyDeleted);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::CategoryDeleted {
            category_id: category_id.to_string(),
            common,
        }])
    }

    fn handle_add_transaction(
        &self,
        transaction_id: TransactionId,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active { id, categories, .. } = self else {
            return Err(AccountError::AccountNotFound);
        };

        if amount.trim().is_empty() {
            return Err(AccountError::EmptyAmount);
        }

        // カテゴリの存在確認（削除されていても既存の取引では使用可能）
        categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        // 簡易的な日付形式チェック
        if !date.contains('-') {
            return Err(AccountError::InvalidDateFormat);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::TransactionAdded {
            transaction_id: transaction_id.to_string(),
            props: TransactionProps {
                amount,
                category_id: category_id.to_string(),
                comment,
                date,
            },
            common,
        }])
    }

    fn handle_update_transaction(
        &self,
        transaction_id: TransactionId,
        amount: String,
        category_id: CategoryId,
        comment: String,
        date: String,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active {
            id,
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

        if amount.trim().is_empty() {
            return Err(AccountError::EmptyAmount);
        }

        // カテゴリの存在確認
        categories
            .get(&category_id)
            .ok_or(AccountError::CategoryNotFound)?;

        // 簡易的な日付形式チェック
        if !date.contains('-') {
            return Err(AccountError::InvalidDateFormat);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::TransactionUpdated {
            transaction_id: transaction_id.to_string(),
            props: TransactionProps {
                amount,
                category_id: category_id.to_string(),
                comment,
                date,
            },
            common,
        }])
    }

    fn handle_delete_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<Vec<AccountEvent>, AccountError> {
        let Account::Active {
            id, transactions, ..
        } = self
        else {
            return Err(AccountError::AccountNotFound);
        };

        if !transactions.contains_key(&transaction_id) {
            return Err(AccountError::TransactionNotFound);
        }

        let common = Self::create_common_props(id);
        Ok(vec![AccountEvent::TransactionDeleted {
            transaction_id: transaction_id.to_string(),
            common,
        }])
    }

    // ヘルパーメソッド

    fn create_common_props(account_id: &AccountId) -> AccountEventCommonProps {
        AccountEventCommonProps {
            account_id: account_id.to_string(),
            at: chrono::Utc::now().to_rfc3339(),
            id: uuid::Uuid::new_v4().to_string(),
            protocol_version: 1,
        }
    }
}

// AccountEvent にヘルパーメソッドを追加
impl AccountEvent {
    pub fn account_id(&self) -> &String {
        match self {
            AccountEvent::AccountCreated { common, .. } => &common.account_id,
            AccountEvent::AccountDeleted { common, .. } => &common.account_id,
            AccountEvent::AccountUpdated { common, .. } => &common.account_id,
            AccountEvent::CategoryAdded { common, .. } => &common.account_id,
            AccountEvent::CategoryDeleted { common, .. } => &common.account_id,
            AccountEvent::CategoryUpdated { common, .. } => &common.account_id,
            AccountEvent::OwnerAdded { common, .. } => &common.account_id,
            AccountEvent::OwnerRemoved { common, .. } => &common.account_id,
            AccountEvent::TransactionAdded { common, .. } => &common.account_id,
            AccountEvent::TransactionDeleted { common, .. } => &common.account_id,
            AccountEvent::TransactionUpdated { common, .. } => &common.account_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account() -> anyhow::Result<()> {
        let account = Account::new();
        let account_id = AccountId::new();
        let command = AccountCommand::CreateAccount {
            account_id,
            name: "My Account".to_string(),
            owners: vec!["user-1".to_string()],
        };

        let events = account.handle_command(command)?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            AccountEvent::AccountCreated { name, owners, .. } => {
                assert_eq!(name, "My Account");
                assert_eq!(owners, &vec!["user-1".to_string()]);
                Ok(())
            }
            event => anyhow::bail!("Expected AccountCreated event, got {:?}", event),
        }
    }

    #[test]
    fn test_account_from_events() -> anyhow::Result<()> {
        let account_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let common = AccountEventCommonProps {
            account_id: account_uuid.to_string(),
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-1".to_string(),
            protocol_version: 1,
        };

        let events = vec![AccountEvent::AccountCreated {
            name: "My Account".to_string(),
            owners: vec!["user-1".to_string()],
            common,
        }];

        let account = Account::from_events(events);
        match account {
            Account::Active {
                id, name, owners, ..
            } => {
                assert_eq!(id.to_string(), account_uuid);
                assert_eq!(name, "My Account");
                assert!(owners.contains("user-1"));
                Ok(())
            }
            Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
        }
    }

    #[test]
    fn test_add_owner() -> anyhow::Result<()> {
        let account_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let common = AccountEventCommonProps {
            account_id: account_uuid.to_string(),
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-1".to_string(),
            protocol_version: 1,
        };

        let mut account = Account::new();
        account.apply_event(&AccountEvent::AccountCreated {
            name: "My Account".to_string(),
            owners: vec!["user-1".to_string()],
            common,
        });

        let command = AccountCommand::AddOwner {
            owner: "user-2".to_string(),
        };

        let events = account.handle_command(command)?;
        assert_eq!(events.len(), 1);

        account.apply_event(&events[0]);
        match account {
            Account::Active { owners, .. } => {
                assert!(owners.contains("user-2"));
                Ok(())
            }
            Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
        }
    }

    #[test]
    fn test_cannot_remove_last_owner() -> anyhow::Result<()> {
        let account_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let common = AccountEventCommonProps {
            account_id: account_uuid.to_string(),
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-1".to_string(),
            protocol_version: 1,
        };

        let mut account = Account::new();
        account.apply_event(&AccountEvent::AccountCreated {
            name: "My Account".to_string(),
            owners: vec!["user-1".to_string()],
            common,
        });

        let command = AccountCommand::RemoveOwner {
            owner: "user-1".to_string(),
        };

        let result = account.handle_command(command);
        match result {
            Err(AccountError::CannotRemoveLastOwner) => Ok(()),
            Ok(_) => anyhow::bail!("Expected CannotRemoveLastOwner error, but command succeeded"),
            Err(e) => anyhow::bail!("Expected CannotRemoveLastOwner error, got {:?}", e),
        }
    }
}
