use crate::schema::QueryAccountMonthlySummaryDocumentData;
use crate::schema::QueryAccountTransactionDocumentData;
use crate::schema::QueryUserDocumentData;
use application::error::ApplicationError;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::MonthlySummaryProjection;
use application::projection::TransactionProjection;
use application::view::AccountView;
use application::view::CategoryView;
use application::view::MonthlySummaryView;
use application::view::PaginatedList;
use application::view::TransactionView;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;
use domain::TransactionId;
use domain::UserId;

impl From<QueryAccountTransactionDocumentData> for TransactionView {
    fn from(data: QueryAccountTransactionDocumentData) -> Self {
        TransactionView {
            id: data.id,
            account_id: data.account_id,
            amount: data.amount,
            category_id: data.category_id,
            date: data.date,
            comment: data.comment,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
    }
}

/// Internal error type for FirestoreProjection operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("account not found: {0}")]
    AccountNotFound(AccountId),

    #[error("deserialize account for account {0}")]
    DeserializeAccount(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("deserialize event for account {0}")]
    DeserializeEvent(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("deserialize user for user {0}")]
    DeserializeUser(UserId, #[source] bouzuya_firestore_client::Error),

    #[error("event not found for account {0}")]
    EventNotFound(AccountId),

    #[error("get account document for account {0}")]
    GetAccountDocument(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("get all event documents for account {0}")]
    GetAllEventDocuments(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("get user document for user {0}")]
    GetUserDocument(UserId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid account document path for account {0}")]
    InvalidAccountDocumentPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid user document path: {0}")]
    InvalidUserDocumentPath(UserId, #[source] bouzuya_firestore_client::Error),

    #[error("list event documents for account {0}")]
    ListEventDocuments(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("get monthly summary for account {0}")]
    GetMonthlySummary(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("deserialize monthly summary for account {0}")]
    DeserializeMonthlySummary(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid monthly summary document path for account {0}")]
    InvalidMonthlySummaryDocumentPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid transaction document path for account {0}")]
    InvalidQueryTransactionDocumentPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("get transaction document for account {0}")]
    GetQueryTransactionDocument(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("deserialize transaction document for account {0}")]
    DeserializeQueryTransactionDocument(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid transactions collection path for account {0}")]
    InvalidQueryTransactionsCollectionPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("query transactions for account {0}")]
    QueryTransactions(AccountId, #[source] bouzuya_firestore_client::Error),
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
    firestore: Firestore,
}

impl FirestoreProjection {
    /// Create a new FirestoreProjection with the given Firestore instance
    pub fn new(firestore: Firestore) -> Self {
        Self { firestore }
    }

    /// Get the path to an account document: `accounts/{accountId}`
    fn account_document_path(account_id: &AccountId) -> String {
        format!("accounts/{}", account_id)
    }

    /// 月別サマリードキュメントのパス: `accounts/{account_id}/stats/monthly`
    fn monthly_summary_document_path(account_id: &AccountId) -> String {
        format!("accounts/{}/stats/monthly", account_id)
    }

    /// Get the path to the events collection: `accounts/{accountId}/events`
    fn events_collection_path(account_id: &AccountId) -> String {
        format!("accounts/{}/events", account_id)
    }

    /// 取引クエリ用ドキュメントのパス: `accounts/{account_id}/transactions/{transaction_id}`
    fn query_transaction_document_path(
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> String {
        format!("accounts/{}/transactions/{}", account_id, transaction_id)
    }

    /// 取引クエリ用コレクションのパス: `accounts/{account_id}/transactions`
    fn query_transactions_collection_path(account_id: &AccountId) -> String {
        format!("accounts/{}/transactions", account_id)
    }

    /// Get the path to a user document: `users/{uid}`
    fn user_path(uid: &str) -> String {
        format!("users/{}", uid)
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
        let collection_path = Self::events_collection_path(account_id);
        let collection_ref = self
            .firestore
            .collection(collection_path)
            .expect("invalid collection path");
        let document_refs = collection_ref
            .list_documents()
            .await
            .map_err(|e| E::ListEventDocuments(*account_id, e))?;

        let snapshots = self
            .firestore
            .get_all(document_refs)
            .await
            .map_err(|e| E::GetAllEventDocuments(*account_id, e))?;

        let mut all_events = Vec::new();
        for snapshot in snapshots {
            let event = snapshot
                .data::<AccountEvent>()
                .ok_or_else(|| E::EventNotFound(*account_id))?
                .map_err(|e| E::DeserializeEvent(*account_id, e))?;
            all_events.push(event);
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

    async fn list_account_owner_ids(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<String>, ApplicationError> {
        let document_ref = self
            .firestore
            .doc(Self::account_document_path(account_id))
            .map_err(|e| E::InvalidAccountDocumentPath(*account_id, e))?;
        let snapshot = document_ref
            .get()
            .await
            .map_err(|e| E::GetAccountDocument(*account_id, e))?;
        let account_doc = snapshot
            .data::<crate::schema::QueryAccountDocumentData>()
            .transpose()
            .map_err(|e| E::DeserializeAccount(*account_id, e))?
            .ok_or_else(|| E::AccountNotFound(*account_id))?;

        Ok(account_doc.owners)
    }

    async fn list_accounts(&self, owner_id: &UserId) -> Result<Vec<AccountView>, ApplicationError> {
        // Get user document to find account IDs
        let user_path = Self::user_path(&owner_id.to_string());
        let document_ref = self
            .firestore
            .doc(user_path)
            .map_err(|e| E::InvalidUserDocumentPath(*owner_id, e))?;
        let snapshot = document_ref
            .get()
            .await
            .map_err(|e| E::GetUserDocument(*owner_id, e))?;

        let account_ids = snapshot
            .data::<QueryUserDocumentData>()
            .transpose()
            .map_err(|e| E::DeserializeUser(*owner_id, e))?
            .map(|data| data.account_ids)
            .unwrap_or_default();

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
        // cursor が指定されていれば、その doc から (date, id) を取得して start_after に使う。
        // cursor の doc が消えていた場合は先頭から (None)。
        let start_after: Option<(String, String)> = match cursor {
            None => None,
            Some(cursor_tx_id) => {
                let cursor_id: TransactionId = cursor_tx_id.parse().map_err(|_| {
                    ApplicationError::InvalidRequest("invalid cursor: not a transaction id".into())
                })?;
                let cursor_path = Self::query_transaction_document_path(account_id, &cursor_id);
                let cursor_ref = self
                    .firestore
                    .doc(cursor_path)
                    .map_err(|e| E::InvalidQueryTransactionDocumentPath(*account_id, e))?;
                let cursor_snapshot = cursor_ref
                    .get()
                    .await
                    .map_err(|e| E::GetQueryTransactionDocument(*account_id, e))?;
                cursor_snapshot
                    .data::<QueryAccountTransactionDocumentData>()
                    .transpose()
                    .map_err(|e| E::DeserializeQueryTransactionDocument(*account_id, e))?
                    .map(|data| (data.date, data.id))
            }
        };

        let collection_path = Self::query_transactions_collection_path(account_id);
        let collection_ref = self
            .firestore
            .collection(collection_path)
            .map_err(|e| E::InvalidQueryTransactionsCollectionPath(*account_id, e))?;

        let query = collection_ref
            .order_by("date", "desc")
            .map_err(|e| E::QueryTransactions(*account_id, e))?
            .order_by("id", "desc")
            .map_err(|e| E::QueryTransactions(*account_id, e))?;
        let query = match start_after {
            Some((date, id)) => query
                .start_after([date, id])
                .map_err(|e| E::QueryTransactions(*account_id, e))?,
            None => query,
        };
        let query = query
            .limit(i32::try_from(limit + 1).expect("limit overflow"))
            .map_err(|e| E::QueryTransactions(*account_id, e))?;
        let snapshot = query
            .get()
            .await
            .map_err(|e| E::QueryTransactions(*account_id, e))?;

        let mut items: Vec<TransactionView> = snapshot
            .into_iter()
            .map(|d| {
                d.data::<QueryAccountTransactionDocumentData>()
                    .map(Into::into)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| E::DeserializeQueryTransactionDocument(*account_id, e))?;

        let has_more = items.len() > limit;
        items.truncate(limit);
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
        let path = Self::query_transaction_document_path(account_id, transaction_id);
        let document_ref = self
            .firestore
            .doc(path)
            .map_err(|e| E::InvalidQueryTransactionDocumentPath(*account_id, e))?;
        let snapshot = document_ref
            .get()
            .await
            .map_err(|e| E::GetQueryTransactionDocument(*account_id, e))?;
        Ok(snapshot
            .data::<QueryAccountTransactionDocumentData>()
            .transpose()
            .map_err(|e| E::DeserializeQueryTransactionDocument(*account_id, e))?
            .map(TransactionView::from))
    }

    async fn list_transactions_for_month(
        &self,
        account_id: &AccountId,
        year: i32,
        month: u32,
    ) -> Result<Vec<TransactionView>, ApplicationError> {
        // [YYYY-MM-01, YYYY-(M+1)-01) の範囲で date を絞る (12 月は翌年 1 月へ繰り上げ)
        let prefix_start = format!("{:04}-{:02}-01", year, month);
        let prefix_end = if month == 12 {
            format!("{:04}-01-01", year + 1)
        } else {
            format!("{:04}-{:02}-01", year, month + 1)
        };

        let collection_path = Self::query_transactions_collection_path(account_id);
        let collection_ref = self
            .firestore
            .collection(collection_path)
            .map_err(|e| E::InvalidQueryTransactionsCollectionPath(*account_id, e))?;

        let snapshot = collection_ref
            .r#where(("date", ">=", prefix_start))
            .map_err(|e| E::QueryTransactions(*account_id, e))?
            .r#where(("date", "<", prefix_end))
            .map_err(|e| E::QueryTransactions(*account_id, e))?
            .order_by("date", "asc")
            .map_err(|e| E::QueryTransactions(*account_id, e))?
            .get()
            .await
            .map_err(|e| E::QueryTransactions(*account_id, e))?;

        let items: Vec<TransactionView> = snapshot
            .into_iter()
            .map(|d| {
                d.data::<QueryAccountTransactionDocumentData>()
                    .map(Into::into)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| E::DeserializeQueryTransactionDocument(*account_id, e))?;

        Ok(items)
    }
}

#[async_trait]
impl MonthlySummaryProjection for FirestoreProjection {
    async fn get_monthly_summary(
        &self,
        account_id: &AccountId,
    ) -> Result<MonthlySummaryView, ApplicationError> {
        let document_reference = self
            .firestore
            .doc(Self::monthly_summary_document_path(account_id))
            .map_err(|e| E::InvalidMonthlySummaryDocumentPath(*account_id, e))?;
        let document_snapshot = document_reference
            .get()
            .await
            .map_err(|e| E::GetMonthlySummary(*account_id, e))?;

        let data = document_snapshot
            .data::<QueryAccountMonthlySummaryDocumentData>()
            .transpose()
            .map_err(|e| E::DeserializeMonthlySummary(*account_id, e))?
            .unwrap_or_default();

        Ok(MonthlySummaryView {
            account_id: account_id.to_string(),
            expenses: data.expenses,
            incomes: data.incomes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FirestoreAccountRepository;
    use application::repository::AccountRepository;
    use bouzuya_firestore_client::FirestoreOptions;
    use domain::AccountEventCommonProps;
    use domain::AccountEventTransactionProps;

    fn setup() -> anyhow::Result<(FirestoreAccountRepository, FirestoreProjection)> {
        let firestore = Firestore::new(FirestoreOptions {
            database_id: None,
            project_id: Some("demo-project".to_string()),
        })?;
        let repo = FirestoreAccountRepository::new(firestore.clone());
        let projection = FirestoreProjection::new(firestore);
        Ok((repo, projection))
    }

    fn account_created_event(account_id: &AccountId, event_id: &str, at: &str) -> AccountEvent {
        AccountEvent::AccountCreated {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: at.to_string(),
                id: event_id.to_string(),
                protocol_version: 3,
            },
            name: "テストアカウント".to_string(),
            owners: vec![],
        }
    }

    fn transaction_added_event(
        account_id: &AccountId,
        event_id: &str,
        at: &str,
        transaction_id: &TransactionId,
        amount: &str,
        date: &str,
        comment: &str,
    ) -> AccountEvent {
        AccountEvent::TransactionAdded {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: at.to_string(),
                id: event_id.to_string(),
                protocol_version: 3,
            },
            props: AccountEventTransactionProps {
                amount: amount.to_string(),
                category_id: "00000000-0000-0000-0000-000000000001".to_string(),
                comment: comment.to_string(),
                date: date.to_string(),
            },
            transaction_id: transaction_id.to_string(),
        }
    }

    /// AccountCreated を保存して、続けて指定された (tx_id, date) 列で TransactionAdded を保存する。
    /// 各 save_events 呼び出しには「直前までに保存した全イベントから再構築した集約」を渡す。
    async fn seed_transactions(
        repo: &FirestoreAccountRepository,
        account_id: &AccountId,
        transactions: &[(TransactionId, &str)],
    ) -> anyhow::Result<()> {
        let mut all_events: Vec<AccountEvent> = vec![];

        let created = account_created_event(account_id, "evt-create", "2024-01-01T00:00:00Z");
        let aggregate_before = Account::from_events(all_events.clone());
        AccountRepository::save_events(repo, account_id, vec![created.clone()], &aggregate_before)
            .await?;
        all_events.push(created);

        for (i, (tx_id, date)) in transactions.iter().enumerate() {
            let added = transaction_added_event(
                account_id,
                &format!("evt-tx-{:03}", i),
                &format!("2024-01-01T00:00:{:02}Z", i + 1),
                tx_id,
                "-1000",
                date,
                "",
            );
            let aggregate_before = Account::from_events(all_events.clone());
            AccountRepository::save_events(
                repo,
                account_id,
                vec![added.clone()],
                &aggregate_before,
            )
            .await?;
            all_events.push(added);
        }
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_get_transaction_returns_view() -> anyhow::Result<()> {
        let (repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_id = TransactionId::generate();

        seed_transactions(&repo, &account_id, &[(tx_id, "2024-01-15")]).await?;

        let view = TransactionProjection::get_transaction(&projection, &account_id, &tx_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("expected transaction"))?;
        assert_eq!(view.id, tx_id.to_string());
        assert_eq!(view.date, "2024-01-15");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_get_transaction_returns_none_when_missing() -> anyhow::Result<()> {
        let (_repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_id = TransactionId::generate();

        let view = TransactionProjection::get_transaction(&projection, &account_id, &tx_id).await?;
        assert!(view.is_none());
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_list_transactions_empty() -> anyhow::Result<()> {
        let (_repo, projection) = setup()?;
        let account_id = AccountId::generate();

        let result =
            TransactionProjection::list_transactions(&projection, &account_id, None, 20).await?;
        assert!(result.items.is_empty());
        assert!(result.next_cursor.is_none());
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_list_transactions_sorted_desc_with_pagination() -> anyhow::Result<()> {
        let (repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_a = TransactionId::generate();
        let tx_b = TransactionId::generate();
        let tx_c = TransactionId::generate();

        // 日付順 (古→新): A=01-10, B=01-20, C=02-05
        seed_transactions(
            &repo,
            &account_id,
            &[
                (tx_a, "2024-01-10"),
                (tx_b, "2024-01-20"),
                (tx_c, "2024-02-05"),
            ],
        )
        .await?;

        // limit=2 で 1 ページ目を取得: 新しい順 C, B が返る。next_cursor は B
        let page1 =
            TransactionProjection::list_transactions(&projection, &account_id, None, 2).await?;
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.items[0].id, tx_c.to_string());
        assert_eq!(page1.items[1].id, tx_b.to_string());
        assert_eq!(
            page1.next_cursor.as_deref(),
            Some(tx_b.to_string().as_str())
        );

        // 2 ページ目: cursor = B → 残り A だけ。next_cursor は None
        let page2 = TransactionProjection::list_transactions(
            &projection,
            &account_id,
            page1.next_cursor,
            2,
        )
        .await?;
        assert_eq!(page2.items.len(), 1);
        assert_eq!(page2.items[0].id, tx_a.to_string());
        assert!(page2.next_cursor.is_none());
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_list_transactions_cursor_missing_returns_from_start() -> anyhow::Result<()> {
        let (repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_a = TransactionId::generate();
        let missing_cursor = TransactionId::generate();

        seed_transactions(&repo, &account_id, &[(tx_a, "2024-01-10")]).await?;

        // 存在しない cursor は先頭から扱い
        let page = TransactionProjection::list_transactions(
            &projection,
            &account_id,
            Some(missing_cursor.to_string()),
            10,
        )
        .await?;
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, tx_a.to_string());
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_list_transactions_for_month_filters_and_sorts_asc() -> anyhow::Result<()> {
        let (repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_in_a = TransactionId::generate();
        let tx_in_b = TransactionId::generate();
        let tx_out_prev = TransactionId::generate();
        let tx_out_next = TransactionId::generate();

        seed_transactions(
            &repo,
            &account_id,
            &[
                (tx_out_prev, "2023-12-31"),
                (tx_in_a, "2024-01-20"),
                (tx_in_b, "2024-01-05"),
                (tx_out_next, "2024-02-01"),
            ],
        )
        .await?;

        let items =
            TransactionProjection::list_transactions_for_month(&projection, &account_id, 2024, 1)
                .await?;
        // 月内のみ、日付昇順
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, tx_in_b.to_string()); // 01-05
        assert_eq!(items[1].id, tx_in_a.to_string()); // 01-20
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_list_transactions_for_month_year_boundary() -> anyhow::Result<()> {
        let (repo, projection) = setup()?;
        let account_id = AccountId::generate();
        let tx_dec = TransactionId::generate();
        let tx_jan = TransactionId::generate();

        seed_transactions(
            &repo,
            &account_id,
            &[(tx_dec, "2024-12-31"), (tx_jan, "2025-01-01")],
        )
        .await?;

        // 12 月クエリ: 12-31 のみ
        let december =
            TransactionProjection::list_transactions_for_month(&projection, &account_id, 2024, 12)
                .await?;
        assert_eq!(december.len(), 1);
        assert_eq!(december[0].id, tx_dec.to_string());

        // 翌 1 月クエリ: 01-01 のみ
        let january =
            TransactionProjection::list_transactions_for_month(&projection, &account_id, 2025, 1)
                .await?;
        assert_eq!(january.len(), 1);
        assert_eq!(january[0].id, tx_jan.to_string());
        Ok(())
    }
}
