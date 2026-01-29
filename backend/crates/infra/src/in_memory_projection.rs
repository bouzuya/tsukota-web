use std::collections::HashMap;
use std::sync::Arc;

use application::error::ApplicationError;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::view::AccountView;
use application::view::CategoryView;
use application::view::PaginatedList;
use application::view::TransactionView;
use async_trait::async_trait;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;
use domain::TransactionId;
use domain::UserId;
use tokio::sync::RwLock;

/// In-memory projection for development and testing
///
/// This projection stores events and rebuilds views on demand.
#[derive(Clone, Default)]
pub struct InMemoryProjection {
    events: Arc<RwLock<HashMap<String, Vec<AccountEvent>>>>,
}

impl InMemoryProjection {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Share the event storage with the event store
    pub fn with_events(events: Arc<RwLock<HashMap<String, Vec<AccountEvent>>>>) -> Self {
        Self { events }
    }

    fn get_event_timestamp(event: &AccountEvent) -> &str {
        match event {
            AccountEvent::AccountCreated { common, .. }
            | AccountEvent::AccountDeleted { common }
            | AccountEvent::AccountUpdated { common, .. }
            | AccountEvent::CategoryAdded { common, .. }
            | AccountEvent::CategoryDeleted { common, .. }
            | AccountEvent::CategoryUpdated { common, .. }
            | AccountEvent::OwnerAdded { common, .. }
            | AccountEvent::OwnerRemoved { common, .. }
            | AccountEvent::TransactionAdded { common, .. }
            | AccountEvent::TransactionDeleted { common, .. }
            | AccountEvent::TransactionUpdated { common, .. } => &common.at,
        }
    }

    fn extract_timestamps(events: &[AccountEvent]) -> (String, String) {
        let created_at = events
            .first()
            .map(|e| Self::get_event_timestamp(e).to_string())
            .unwrap_or_default();
        let updated_at = events
            .last()
            .map(|e| Self::get_event_timestamp(e).to_string())
            .unwrap_or_default();
        (created_at, updated_at)
    }

    fn build_account_view(
        account_id: &str,
        account: &Account,
        events: &[AccountEvent],
    ) -> Option<AccountView> {
        match account {
            Account::Active { name, owners, .. } => {
                let (created_at, updated_at) = Self::extract_timestamps(events);
                Some(AccountView {
                    id: account_id.to_string(),
                    name: name.clone(),
                    owner_ids: owners.iter().map(|o| o.to_string()).collect(),
                    created_at,
                    updated_at,
                })
            }
            Account::Empty => None,
        }
    }

    fn build_category_views(
        account_id: &str,
        account: &Account,
        events: &[AccountEvent],
    ) -> Vec<CategoryView> {
        match account {
            Account::Active { categories, .. } => categories
                .values()
                .map(|cat| {
                    // Find CategoryAdded event for created_at
                    let created_at = events
                        .iter()
                        .find_map(|e| match e {
                            AccountEvent::CategoryAdded {
                                category_id,
                                common,
                                ..
                            } if category_id == &cat.id.to_string() => Some(common.at.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();

                    // Find CategoryDeleted event for deleted_at
                    let deleted_at = if cat.deleted {
                        events.iter().find_map(|e| match e {
                            AccountEvent::CategoryDeleted {
                                category_id,
                                common,
                            } if category_id == &cat.id.to_string() => Some(common.at.clone()),
                            _ => None,
                        })
                    } else {
                        None
                    };

                    CategoryView {
                        id: cat.id.to_string(),
                        account_id: account_id.to_string(),
                        name: cat.name.clone(),
                        created_at,
                        deleted_at,
                    }
                })
                .collect(),
            Account::Empty => vec![],
        }
    }

    fn build_transaction_views(
        account_id: &str,
        account: &Account,
        events: &[AccountEvent],
    ) -> Vec<TransactionView> {
        match account {
            Account::Active { transactions, .. } => transactions
                .values()
                .map(|tx| {
                    // Find TransactionAdded event for created_at
                    let created_at = events
                        .iter()
                        .find_map(|e| match e {
                            AccountEvent::TransactionAdded {
                                transaction_id,
                                common,
                                ..
                            } if transaction_id == &tx.id.to_string() => Some(common.at.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();

                    // Find latest TransactionUpdated or use created_at for updated_at
                    let updated_at = events
                        .iter()
                        .rev()
                        .find_map(|e| match e {
                            AccountEvent::TransactionUpdated {
                                transaction_id,
                                common,
                                ..
                            } if transaction_id == &tx.id.to_string() => Some(common.at.clone()),
                            _ => None,
                        })
                        .unwrap_or_else(|| created_at.clone());

                    TransactionView {
                        id: tx.id.to_string(),
                        account_id: account_id.to_string(),
                        amount: tx.amount.clone(),
                        category_id: tx.category_id.to_string(),
                        date: tx.date.clone(),
                        comment: tx.comment.clone(),
                        created_at,
                        updated_at,
                    }
                })
                .collect(),
            Account::Empty => vec![],
        }
    }
}

#[async_trait]
impl AccountProjection for InMemoryProjection {
    async fn get_account(
        &self,
        account_id: &AccountId,
    ) -> Result<Option<AccountView>, ApplicationError> {
        let events_map = self.events.read().await;
        let account_events = events_map.get(&account_id.to_string());

        match account_events {
            Some(events) if !events.is_empty() => {
                let account = Account::from_events(events.clone());
                Ok(Self::build_account_view(
                    &account_id.to_string(),
                    &account,
                    events,
                ))
            }
            _ => Ok(None),
        }
    }

    async fn list_accounts(&self, owner_id: &UserId) -> Result<Vec<AccountView>, ApplicationError> {
        let events_map = self.events.read().await;
        let mut accounts = Vec::new();

        for (account_id, account_events) in events_map.iter() {
            if account_events.is_empty() {
                continue;
            }
            let account = Account::from_events(account_events.clone());
            if let Account::Active { owners, .. } = &account
                && owners.contains(owner_id)
                && let Some(view) = Self::build_account_view(account_id, &account, account_events)
            {
                accounts.push(view);
            }
        }

        Ok(accounts)
    }
}

#[async_trait]
impl CategoryProjection for InMemoryProjection {
    async fn list_categories(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<CategoryView>, ApplicationError> {
        let events_map = self.events.read().await;
        let account_events = events_map.get(&account_id.to_string());

        match account_events {
            Some(events) if !events.is_empty() => {
                let account = Account::from_events(events.clone());
                Ok(Self::build_category_views(
                    &account_id.to_string(),
                    &account,
                    events,
                ))
            }
            _ => Ok(vec![]),
        }
    }
}

#[async_trait]
impl TransactionProjection for InMemoryProjection {
    async fn list_transactions(
        &self,
        account_id: &AccountId,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<PaginatedList<TransactionView>, ApplicationError> {
        let events_map = self.events.read().await;
        let account_events = events_map.get(&account_id.to_string());

        let mut transactions = match account_events {
            Some(events) if !events.is_empty() => {
                let account = Account::from_events(events.clone());
                Self::build_transaction_views(&account_id.to_string(), &account, events)
            }
            _ => vec![],
        };

        // Sort by date descending
        transactions.sort_by(|a, b| b.date.cmp(&a.date));

        // Apply cursor-based pagination
        let start_idx = if let Some(cursor) = cursor {
            transactions
                .iter()
                .position(|t| t.id == cursor)
                .map(|i| i + 1)
                .unwrap_or(0)
        } else {
            0
        };

        let paginated: Vec<_> = transactions
            .into_iter()
            .skip(start_idx)
            .take(limit + 1)
            .collect();

        let has_more = paginated.len() > limit;
        let items: Vec<_> = paginated.into_iter().take(limit).collect();
        let next_cursor = if has_more {
            items.last().map(|t| t.id.clone())
        } else {
            None
        };

        Ok(PaginatedList { items, next_cursor })
    }

    async fn get_transaction(
        &self,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> Result<Option<TransactionView>, ApplicationError> {
        let events_map = self.events.read().await;
        let account_events = events_map.get(&account_id.to_string());

        let transactions = match account_events {
            Some(events) if !events.is_empty() => {
                let account = Account::from_events(events.clone());
                Self::build_transaction_views(&account_id.to_string(), &account, events)
            }
            _ => vec![],
        };

        Ok(transactions
            .into_iter()
            .find(|t| t.id == transaction_id.to_string()))
    }

    async fn list_transactions_for_month(
        &self,
        account_id: &AccountId,
        year: i32,
        month: u32,
    ) -> Result<Vec<TransactionView>, ApplicationError> {
        let events_map = self.events.read().await;
        let account_events = events_map.get(&account_id.to_string());

        let mut transactions = match account_events {
            Some(events) if !events.is_empty() => {
                let account = Account::from_events(events.clone());
                Self::build_transaction_views(&account_id.to_string(), &account, events)
            }
            _ => vec![],
        };

        // Filter by year and month (date format: YYYY-MM-DD)
        let prefix = format!("{:04}-{:02}", year, month);
        transactions.retain(|t| t.date.starts_with(&prefix));

        // Sort by date
        transactions.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(transactions)
    }
}
