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
use firestore_client::FirestoreClient;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreProjection operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("firestore client: {0}")]
    FirestoreClient(#[from] firestore_client::FirestoreClientError),
}

impl From<E> for ApplicationError {
    fn from(e: E) -> Self {
        ApplicationError::Repository(e.to_string())
    }
}

/// Firestore-based projection implementation
///
/// This projection reads events from Firestore and rebuilds views on demand.
#[derive(Clone)]
pub struct FirestoreProjection {
    client: FirestoreClient,
}

impl FirestoreProjection {
    /// Create a new FirestoreProjection with the given client
    pub fn new(client: FirestoreClient) -> Self {
        Self { client }
    }

    /// Get the path to the events collection: `accounts/{accountId}/events`
    fn events_collection_path(account_id: &AccountId) -> Result<CollectionPath, E> {
        let path_str = format!("accounts/{}/events", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a user document: `users/{uid}`
    fn user_path(uid: &str) -> Result<DocumentPath, E> {
        let path_str = format!("users/{}", uid);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Extract the `at` timestamp from an AccountEvent
    fn get_event_at(event: &AccountEvent) -> &str {
        match event {
            AccountEvent::AccountCreated { common, .. }
            | AccountEvent::AccountDeleted { common, .. }
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

    /// Load all events for an account from Firestore
    async fn load_events(&self, account_id: &AccountId) -> Result<Vec<AccountEvent>, E> {
        let collection_path = Self::events_collection_path(account_id)?;

        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let response = self
                .client
                .list_documents(collection_path.clone(), page_token)
                .await?;

            for doc in response.documents {
                let event: AccountEvent = self.client.deserialize(doc.fields)?;
                all_events.push(event);
            }

            if response.next_page_token.is_empty() {
                break;
            }
            page_token = Some(response.next_page_token);
        }

        // Sort events by their `at` timestamp to ensure correct ordering
        all_events.sort_by(|a, b| Self::get_event_at(a).cmp(Self::get_event_at(b)));

        Ok(all_events)
    }

    fn extract_timestamps(events: &[AccountEvent]) -> (String, String) {
        let created_at = events
            .first()
            .map(|e| Self::get_event_at(e).to_string())
            .unwrap_or_default();
        let updated_at = events
            .last()
            .map(|e| Self::get_event_at(e).to_string())
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

/// User document schema
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UserDocument {
    id: String,
    account_ids: Vec<String>,
}

#[async_trait]
impl AccountProjection for FirestoreProjection {
    async fn get_account(
        &self,
        account_id: &AccountId,
    ) -> Result<Option<AccountView>, ApplicationError> {
        let events = self.load_events(account_id).await?;

        if events.is_empty() {
            return Ok(None);
        }

        let account = Account::from_events(events.clone());
        Ok(Self::build_account_view(
            &account_id.to_string(),
            &account,
            &events,
        ))
    }

    async fn list_accounts(&self, owner_id: &UserId) -> Result<Vec<AccountView>, ApplicationError> {
        // Get user document to find account IDs
        let user_path = Self::user_path(&owner_id.to_string()).map_err(ApplicationError::from)?;
        let user_doc = self
            .client
            .get_document(user_path)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        let account_ids = match user_doc {
            Some(doc) => {
                let user: UserDocument = self
                    .client
                    .deserialize(doc.fields)
                    .map_err(|e| ApplicationError::Repository(e.to_string()))?;
                user.account_ids
            }
            None => vec![],
        };

        // Load each account
        let mut accounts = Vec::new();
        for account_id_str in account_ids {
            let Ok(account_id) = account_id_str.parse::<AccountId>() else {
                continue;
            };
            let events = self.load_events(&account_id).await?;

            if events.is_empty() {
                continue;
            }

            let account = Account::from_events(events.clone());
            if let Some(view) = Self::build_account_view(&account_id_str, &account, &events) {
                accounts.push(view);
            }
        }

        Ok(accounts)
    }
}

#[async_trait]
impl CategoryProjection for FirestoreProjection {
    async fn list_categories(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<CategoryView>, ApplicationError> {
        let events = self.load_events(account_id).await?;

        if events.is_empty() {
            return Ok(vec![]);
        }

        let account = Account::from_events(events.clone());
        Ok(Self::build_category_views(
            &account_id.to_string(),
            &account,
            &events,
        ))
    }
}

#[async_trait]
impl TransactionProjection for FirestoreProjection {
    async fn list_transactions(
        &self,
        account_id: &AccountId,
        cursor: Option<String>,
        limit: usize,
    ) -> Result<PaginatedList<TransactionView>, ApplicationError> {
        let events = self.load_events(account_id).await?;

        let mut transactions = if events.is_empty() {
            vec![]
        } else {
            let account = Account::from_events(events.clone());
            Self::build_transaction_views(&account_id.to_string(), &account, &events)
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
        let events = self.load_events(account_id).await?;

        let transactions = if events.is_empty() {
            vec![]
        } else {
            let account = Account::from_events(events.clone());
            Self::build_transaction_views(&account_id.to_string(), &account, &events)
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
        let events = self.load_events(account_id).await?;

        let mut transactions = if events.is_empty() {
            vec![]
        } else {
            let account = Account::from_events(events.clone());
            Self::build_transaction_views(&account_id.to_string(), &account, &events)
        };

        // Filter by year and month (date format: YYYY-MM-DD)
        let prefix = format!("{:04}-{:02}", year, month);
        transactions.retain(|t| t.date.starts_with(&prefix));

        // Sort by date
        transactions.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(transactions)
    }
}
