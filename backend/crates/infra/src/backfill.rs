//! ワンショット backfill: events から取引クエリ用ドキュメント
//! (`accounts/{account_id}/transactions/{transaction_id}`) を一括再構築する。
//!
//! 既存アカウント (write 経路の変更前から存在する) に対して read model を初期化するため、
//! 本番反映前に CLI 経由で 1 度だけ実行する想定。Idempotent (再実行しても結果は同じ)。

use crate::FirestoreAccountRepository;
use crate::schema::QueryAccountMonthlySummaryDocumentData;
use crate::schema::QueryAccountTransactionDocumentData;
use application::error::ApplicationError;
use application::repository::AccountRepository;
use bouzuya_firestore_client::Firestore;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;

/// backfill 実行中のエラー
#[derive(Debug, thiserror::Error)]
pub enum BackfillError {
    #[error("invalid event_streams collection path")]
    InvalidEventStreamsCollectionPath(#[source] bouzuya_firestore_client::Error),

    #[error("list event_streams documents")]
    ListEventStreamsDocuments(#[source] bouzuya_firestore_client::Error),

    #[error("load events for account {0}")]
    LoadEvents(AccountId, #[source] ApplicationError),

    #[error("invalid transaction document path for account {0}")]
    InvalidQueryTransactionDocumentPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("set transaction document for account {0}")]
    SetQueryTransactionDocument(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("invalid monthly summary document path for account {0}")]
    InvalidMonthlySummaryDocumentPath(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("set monthly summary document for account {0}")]
    SetMonthlySummaryDocument(AccountId, #[source] bouzuya_firestore_client::Error),
}

/// backfill 実行結果の集計
#[derive(Clone, Debug, Default)]
pub struct BackfillStats {
    /// 走査したアカウント数
    pub accounts_scanned: usize,
    /// AccountId として parse 可能で events を持っていたアカウント数
    pub accounts_processed: usize,
    /// 書き込んだ取引ドキュメント数
    pub transactions_written: usize,
}

/// 月別サマリー backfill 実行結果の集計
#[derive(Clone, Debug, Default)]
pub struct MonthlySummaryBackfillStats {
    /// 走査したアカウント数
    pub accounts_scanned: usize,
    /// AccountId として parse 可能で events を持っていたアカウント数
    pub accounts_processed: usize,
    /// 書き込んだ月別サマリードキュメント数
    pub monthly_summaries_written: usize,
}

/// 全アカウントの取引クエリ用ドキュメントを events から再構築する。
///
/// 1. `aggregates/account/event_streams` 配下を列挙してアカウント ID を取得
/// 2. 各アカウントの events を `AccountRepository::load_events` で取得
/// 3. `Account::from_events` で集約を再構築し、現在 active な transactions について
///    `QueryAccountTransactionDocumentData` を組み立て
/// 4. `accounts/{id}/transactions/{tx_id}` に `set` で書き込み (upsert)
pub async fn backfill_query_transactions(
    firestore: &Firestore,
    account_repository: &FirestoreAccountRepository,
) -> Result<BackfillStats, BackfillError> {
    let event_streams = firestore
        .collection("aggregates/account/event_streams")
        .map_err(BackfillError::InvalidEventStreamsCollectionPath)?
        .list_documents()
        .await
        .map_err(BackfillError::ListEventStreamsDocuments)?;

    let mut stats = BackfillStats::default();
    for stream_ref in event_streams {
        stats.accounts_scanned += 1;

        let id_str = stream_ref.id();
        let account_id: AccountId = match id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                tracing::warn!(id = %id_str, "skipping non-uuid event stream id");
                continue;
            }
        };

        let events = AccountRepository::load_events(account_repository, &account_id)
            .await
            .map_err(|e| BackfillError::LoadEvents(account_id, e))?;
        if events.is_empty() {
            tracing::debug!(account_id = %account_id, "no events; skipping");
            continue;
        }

        let account = Account::from_events(events.clone());
        let docs = build_query_transaction_documents(&account_id, &account, &events);

        for doc_data in &docs {
            let path = format!("accounts/{}/transactions/{}", account_id, doc_data.id);
            firestore
                .doc(path)
                .map_err(|e| BackfillError::InvalidQueryTransactionDocumentPath(account_id, e))?
                .set(doc_data)
                .await
                .map_err(|e| BackfillError::SetQueryTransactionDocument(account_id, e))?;
        }

        stats.accounts_processed += 1;
        stats.transactions_written += docs.len();
        tracing::info!(
            account_id = %account_id,
            written = docs.len(),
            "backfilled transactions",
        );
    }

    Ok(stats)
}

/// 全アカウントの月別サマリードキュメント (`accounts/{id}/stats/monthly`) を
/// events から再構築する。
///
/// 1. `aggregates/account/event_streams` 配下を列挙してアカウント ID を取得
/// 2. 各アカウントの events を `AccountRepository::load_events` で取得
/// 3. `Account::from_events` で集約を再構築し、現在 active な transactions を
///    月キー ("YYYY-MM") で集計
/// 4. `accounts/{id}/stats/monthly` に `set` で書き込み (upsert)
///
/// active な transactions が無いアカウント (集約が `Empty`、または全取引が削除済み)
/// では書き込みを行わない。これは write 経路 (`update_monthly_summary_in_tx`) が
/// 取引イベント発生時にのみサマリードキュメントを生成する挙動と揃えるため。
/// Idempotent (再実行しても結果は同じ)。
pub async fn backfill_monthly_summaries(
    firestore: &Firestore,
    account_repository: &FirestoreAccountRepository,
) -> Result<MonthlySummaryBackfillStats, BackfillError> {
    let event_streams = firestore
        .collection("aggregates/account/event_streams")
        .map_err(BackfillError::InvalidEventStreamsCollectionPath)?
        .list_documents()
        .await
        .map_err(BackfillError::ListEventStreamsDocuments)?;

    let mut stats = MonthlySummaryBackfillStats::default();
    for stream_ref in event_streams {
        stats.accounts_scanned += 1;

        let id_str = stream_ref.id();
        let account_id: AccountId = match id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                tracing::warn!(id = %id_str, "skipping non-uuid event stream id");
                continue;
            }
        };

        let events = AccountRepository::load_events(account_repository, &account_id)
            .await
            .map_err(|e| BackfillError::LoadEvents(account_id, e))?;
        if events.is_empty() {
            tracing::debug!(account_id = %account_id, "no events; skipping");
            continue;
        }
        stats.accounts_processed += 1;

        let account = Account::from_events(events.clone());
        let summary = build_monthly_summary_document(&account_id, &account);

        // active な取引が 1 件も無い場合はサマリードキュメントを作らない。
        if summary.incomes.is_empty() && summary.expenses.is_empty() {
            tracing::debug!(account_id = %account_id, "no active transactions; skipping summary");
            continue;
        }

        let path = format!("accounts/{}/stats/monthly", account_id);
        firestore
            .doc(path)
            .map_err(|e| BackfillError::InvalidMonthlySummaryDocumentPath(account_id, e))?
            .set(&summary)
            .await
            .map_err(|e| BackfillError::SetMonthlySummaryDocument(account_id, e))?;

        stats.monthly_summaries_written += 1;
        tracing::info!(
            account_id = %account_id,
            months_incomes = summary.incomes.len(),
            months_expenses = summary.expenses.len(),
            "backfilled monthly summary",
        );
    }

    Ok(stats)
}

/// 現在 active な transactions について、events から created_at / updated_at を補完して
/// `QueryAccountTransactionDocumentData` の列を作る。
///
/// - `created_at`: 該当 `transaction_id` の `TransactionAdded` event の `at`
/// - `updated_at`: 最後の `TransactionUpdated` event の `at` (なければ `created_at`)
///
/// `Account::Empty` の場合は空 vec を返す。削除済み transaction は `Account::Active.transactions`
/// に存在しないため自然に除外される。
fn build_query_transaction_documents(
    account_id: &AccountId,
    account: &Account,
    events: &[AccountEvent],
) -> Vec<QueryAccountTransactionDocumentData> {
    let transactions = match account {
        Account::Active { transactions, .. } => transactions,
        Account::Empty => return vec![],
    };

    transactions
        .values()
        .map(|tx| {
            let tx_id_str = tx.id.to_string();
            let created_at = events
                .iter()
                .find_map(|e| match e {
                    AccountEvent::TransactionAdded {
                        transaction_id,
                        common,
                        ..
                    } if transaction_id == &tx_id_str => Some(common.at.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let updated_at = events
                .iter()
                .rev()
                .find_map(|e| match e {
                    AccountEvent::TransactionUpdated {
                        transaction_id,
                        common,
                        ..
                    } if transaction_id == &tx_id_str => Some(common.at.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| created_at.clone());
            QueryAccountTransactionDocumentData {
                account_id: account_id.to_string(),
                amount: tx.amount.clone(),
                category_id: tx.category_id.to_string(),
                comment: tx.comment.clone(),
                created_at,
                date: tx.date.clone(),
                id: tx_id_str,
                updated_at,
            }
        })
        .collect()
}

/// 現在 active な transactions を月キー ("YYYY-MM") で集計し、
/// `QueryAccountMonthlySummaryDocumentData` を組み立てる。
///
/// 金額が 0 以上なら `incomes`、負なら `expenses` バケットに月ごとに合算する
/// (write 経路 `update_monthly_summary_in_tx` と同じ分類規則)。金額・月キーの
/// 抽出も write 経路と揃える (date は "YYYY-MM-DD" 前提で先頭 7 文字を月キーに、
/// 金額は i64 へパースし不正値は 0 とみなす)。`Account::Empty` の場合は
/// `id` のみ持つ空サマリーを返す。
fn build_monthly_summary_document(
    account_id: &AccountId,
    account: &Account,
) -> QueryAccountMonthlySummaryDocumentData {
    let mut summary = QueryAccountMonthlySummaryDocumentData {
        id: account_id.to_string(),
        ..QueryAccountMonthlySummaryDocumentData::default()
    };

    let transactions = match account {
        Account::Active { transactions, .. } => transactions,
        Account::Empty => return summary,
    };

    for tx in transactions.values() {
        let month_key = tx.date[..7].to_string();
        let amount = tx.amount.parse::<i64>().unwrap_or(0);
        // incomes (>= 0) / expenses (< 0)
        let bucket = if amount >= 0 {
            &mut summary.incomes
        } else {
            &mut summary.expenses
        };
        let current = bucket
            .get(&month_key)
            .map(|s| s.parse::<i64>().unwrap_or(0))
            .unwrap_or(0);
        bucket.insert(month_key, (current + amount).to_string());
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use bouzuya_firestore_client::FirestoreOptions;
    use bouzuya_firestore_client::Precondition;
    use domain::AccountEventCommonProps;
    use domain::AccountEventTransactionProps;
    use domain::TransactionId;

    fn setup() -> anyhow::Result<(Firestore, FirestoreAccountRepository)> {
        let firestore = Firestore::new(FirestoreOptions {
            database_id: None,
            project_id: Some("demo-project".to_string()),
        })?;
        let repo = FirestoreAccountRepository::new(firestore.clone());
        Ok((firestore, repo))
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
                comment: String::new(),
                date: date.to_string(),
            },
            transaction_id: transaction_id.to_string(),
        }
    }

    /// AccountCreated + (tx_id, date) ごとの TransactionAdded を保存する
    async fn seed_events(
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

    /// 取引クエリ用ドキュメントを Firestore から読み出す
    async fn read_doc(
        firestore: &Firestore,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> anyhow::Result<Option<QueryAccountTransactionDocumentData>> {
        let path = format!("accounts/{}/transactions/{}", account_id, transaction_id);
        let snapshot = firestore.doc(path)?.get().await?;
        Ok(snapshot
            .data::<QueryAccountTransactionDocumentData>()
            .transpose()?)
    }

    /// 取引クエリ用ドキュメントを削除する
    async fn delete_doc(
        firestore: &Firestore,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> anyhow::Result<()> {
        let path = format!("accounts/{}/transactions/{}", account_id, transaction_id);
        firestore
            .doc(path)?
            .delete(Precondition {
                exists: None,
                last_update_time: None,
            })
            .await?;
        Ok(())
    }

    /// 月別サマリードキュメントを Firestore から読み出す
    async fn read_summary(
        firestore: &Firestore,
        account_id: &AccountId,
    ) -> anyhow::Result<Option<QueryAccountMonthlySummaryDocumentData>> {
        let path = format!("accounts/{}/stats/monthly", account_id);
        let snapshot = firestore.doc(path)?.get().await?;
        Ok(snapshot
            .data::<QueryAccountMonthlySummaryDocumentData>()
            .transpose()?)
    }

    /// 月別サマリードキュメントを削除する
    async fn delete_summary(firestore: &Firestore, account_id: &AccountId) -> anyhow::Result<()> {
        let path = format!("accounts/{}/stats/monthly", account_id);
        firestore
            .doc(path)?
            .delete(Precondition {
                exists: None,
                last_update_time: None,
            })
            .await?;
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_backfill_recreates_deleted_transaction_doc() -> anyhow::Result<()> {
        let (firestore, repo) = setup()?;
        let account_id = AccountId::generate();
        let tx_id = TransactionId::generate();

        // 1. events と projection doc を作成 (write 経路)
        seed_events(&repo, &account_id, &[(tx_id, "2024-01-15")]).await?;
        assert!(
            read_doc(&firestore, &account_id, &tx_id).await?.is_some(),
            "precondition: doc should exist after seed"
        );

        // 2. projection doc を削除して "backfill 前" 状態を模擬
        delete_doc(&firestore, &account_id, &tx_id).await?;
        assert!(
            read_doc(&firestore, &account_id, &tx_id).await?.is_none(),
            "precondition: doc should be gone after delete"
        );

        // 3. backfill を実行
        let stats = backfill_query_transactions(&firestore, &repo).await?;
        assert!(
            stats.transactions_written >= 1,
            "expected at least 1 transaction written, got {}",
            stats.transactions_written
        );

        // 4. doc が events から再生成されていることを確認
        let doc = read_doc(&firestore, &account_id, &tx_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("expected doc to be recreated"))?;
        assert_eq!(doc.id, tx_id.to_string());
        assert_eq!(doc.account_id, account_id.to_string());
        assert_eq!(doc.amount, "-1000");
        assert_eq!(doc.date, "2024-01-15");
        // seed_events の event at = "2024-01-01T00:00:01Z" (i=0)
        assert_eq!(doc.created_at, "2024-01-01T00:00:01Z");
        assert_eq!(doc.updated_at, "2024-01-01T00:00:01Z");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_backfill_is_idempotent() -> anyhow::Result<()> {
        let (firestore, repo) = setup()?;
        let account_id = AccountId::generate();
        let tx_id = TransactionId::generate();

        seed_events(&repo, &account_id, &[(tx_id, "2024-01-15")]).await?;

        // 2 回連続実行してもエラーにならない
        backfill_query_transactions(&firestore, &repo).await?;
        backfill_query_transactions(&firestore, &repo).await?;

        let doc = read_doc(&firestore, &account_id, &tx_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("expected doc"))?;
        assert_eq!(doc.date, "2024-01-15");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_backfill_monthly_summaries_recreates_deleted_summary_doc() -> anyhow::Result<()> {
        let (firestore, repo) = setup()?;
        let account_id = AccountId::generate();
        let tx_id1 = TransactionId::generate();
        let tx_id2 = TransactionId::generate();

        // 1. events と summary doc を作成 (write 経路)。同月の支出 2 件。
        seed_events(
            &repo,
            &account_id,
            &[(tx_id1, "2024-01-15"), (tx_id2, "2024-01-20")],
        )
        .await?;
        let before = read_summary(&firestore, &account_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("precondition: summary should exist after seed"))?;
        assert_eq!(
            before.expenses.get("2024-01").map(String::as_str),
            Some("-2000")
        );

        // 2. summary doc を削除して "backfill 前" 状態を模擬
        delete_summary(&firestore, &account_id).await?;
        assert!(
            read_summary(&firestore, &account_id).await?.is_none(),
            "precondition: summary should be gone after delete"
        );

        // 3. backfill を実行
        let stats = backfill_monthly_summaries(&firestore, &repo).await?;
        assert!(
            stats.monthly_summaries_written >= 1,
            "expected at least 1 summary written, got {}",
            stats.monthly_summaries_written
        );

        // 4. summary が events から再生成されていることを確認
        let after = read_summary(&firestore, &account_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("expected summary to be recreated"))?;
        assert_eq!(after.id, account_id.to_string());
        assert_eq!(
            after.expenses.get("2024-01").map(String::as_str),
            Some("-2000")
        );
        assert!(after.incomes.is_empty(), "no positive amounts were seeded");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_backfill_monthly_summaries_is_idempotent() -> anyhow::Result<()> {
        let (firestore, repo) = setup()?;
        let account_id = AccountId::generate();
        let tx_id = TransactionId::generate();

        seed_events(&repo, &account_id, &[(tx_id, "2024-01-15")]).await?;

        // 2 回連続実行してもエラーにならず、結果も変わらない
        backfill_monthly_summaries(&firestore, &repo).await?;
        backfill_monthly_summaries(&firestore, &repo).await?;

        let summary = read_summary(&firestore, &account_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("expected summary"))?;
        assert_eq!(
            summary.expenses.get("2024-01").map(String::as_str),
            Some("-1000")
        );
        Ok(())
    }
}
