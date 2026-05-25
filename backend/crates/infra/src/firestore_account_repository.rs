use crate::FirestoreUserRepository;
use crate::repository::Repository;
use crate::schema::AccountEventStreamDocumentData;
use crate::schema::QueryAccountDocumentData;
use crate::schema::QueryAccountMonthlySummaryDocumentData;
use crate::schema::QueryAccountTransactionDocumentData;
use application::error::ApplicationError;
use application::repository::AccountRepository;
use application::repository::UserRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use domain::Account;
use domain::AccountEvent;
use domain::AccountId;
use domain::TransactionId;
use domain::User;
use domain::UserCommand;
use domain::UserId;

/// Firestore-based event store implementation
#[derive(Clone)]
pub struct FirestoreAccountRepository {
    firestore: Firestore,
    user_repository: FirestoreUserRepository,
}

impl FirestoreAccountRepository {
    /// Create a new FirestoreAccountRepository with the given firestore instance
    pub fn new(firestore: Firestore) -> Self {
        let user_repository = FirestoreUserRepository::new(firestore.clone());
        Self {
            firestore,
            user_repository,
        }
    }

    /// Get the path to a query account document: `accounts/{account_id}`
    fn query_account_document_path(account_id: &AccountId) -> String {
        format!("accounts/{}", account_id)
    }

    /// Get the path to a query event document: `accounts/{account_id}/events/{event_id}`
    fn query_event_document_path(account_id: &AccountId, event_id: &str) -> String {
        format!("accounts/{}/events/{}", account_id, event_id)
    }

    /// 取引クエリ用ドキュメントのパス: `accounts/{account_id}/transactions/{transaction_id}`
    fn query_transaction_document_path(
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> String {
        format!("accounts/{}/transactions/{}", account_id, transaction_id)
    }
}

impl Repository for FirestoreAccountRepository {
    type Event = AccountEvent;
    type EventAt = String;
    type EventId = String;
    type EventStream = AccountEventStreamDocumentData;
    type EventStreamId = AccountId;

    fn aggregate_name() -> String {
        "account".to_string()
    }

    fn firestore(&self) -> &Firestore {
        &self.firestore
    }

    fn get_event_at(event: &Self::Event) -> Self::EventAt {
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
            | AccountEvent::TransactionUpdated { common, .. } => common.at.clone(),
        }
    }

    fn get_event_id(event: &Self::Event) -> Self::EventId {
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
            | AccountEvent::TransactionUpdated { common, .. } => common.id.clone(),
        }
    }

    fn new_event_stream(
        event_stream_id: &Self::EventStreamId,
        events: &[Self::Event],
        stored_event_stream: Option<Self::EventStream>,
    ) -> Self::EventStream {
        let mut owners = match &stored_event_stream {
            Some(stored) => stored.owners.clone(),
            None => match &events[0] {
                AccountEvent::AccountCreated { owners, .. } => owners.clone(),
                AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::OwnerAdded { .. }
                | AccountEvent::OwnerRemoved { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => vec![],
            },
        };

        // イベントからオーナー変更を適用
        for event in events {
            match event {
                AccountEvent::OwnerAdded { owner, .. } => {
                    if !owners.contains(owner) {
                        owners.push(owner.clone());
                    }
                }
                AccountEvent::OwnerRemoved { owner, .. } => {
                    owners.retain(|o| o != owner);
                }
                AccountEvent::AccountCreated { .. }
                | AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => {}
            }
        }

        let last_event = events.last().expect("events is non-empty");
        AccountEventStreamDocumentData {
            id: event_stream_id.to_string(),
            last_event_id: Self::get_event_id(last_event),
            owners,
            protocol_version: domain::Account::PROTOCOL_VERSION,
            updated_at: Self::get_event_at(last_event),
        }
    }
}

#[async_trait]
impl AccountRepository for FirestoreAccountRepository {
    async fn load_events(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<AccountEvent>, ApplicationError> {
        Repository::load_events(self, account_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn save_events(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
        aggregate: &Account,
    ) -> Result<(), ApplicationError> {
        // オーナー変更を収集 (User 集約の更新に使用)
        let user_updates = Self::collect_user_updates(&events);

        let account_id_owned = *account_id;
        let events_clone = events.clone();
        let firestore = self.firestore.clone();
        let aggregate_clone = aggregate.clone();
        Repository::save_events(
            self,
            *account_id,
            events,
            Box::new(move |transaction| {
                Box::pin(async move {
                    // accounts/* (クエリ用コレクション) への書き込み
                    Self::build_query_account_writes_in_tx(
                        &firestore,
                        &account_id_owned,
                        &events_clone,
                        &aggregate_clone,
                        transaction,
                    )
                    .await?;
                    Ok(())
                })
            }),
        )
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // User 集約の更新 (別トランザクション)
        self.update_user_aggregates(account_id, user_updates)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}

impl FirestoreAccountRepository {
    /// AccountEvent からオーナー変更を収集する
    fn collect_user_updates(events: &[AccountEvent]) -> Vec<(UserId, UserUpdateAction)> {
        let mut updates = Vec::new();

        for event in events {
            match event {
                AccountEvent::AccountCreated { owners, .. } => {
                    for owner in owners {
                        if let Ok(user_id) = owner.parse::<UserId>() {
                            updates.push((user_id, UserUpdateAction::AddAccount));
                        }
                    }
                }
                AccountEvent::OwnerAdded { owner, .. } => {
                    if let Ok(user_id) = owner.parse::<UserId>() {
                        updates.push((user_id, UserUpdateAction::AddAccount));
                    }
                }
                AccountEvent::OwnerRemoved { owner, .. } => {
                    if let Ok(user_id) = owner.parse::<UserId>() {
                        updates.push((user_id, UserUpdateAction::RemoveAccount));
                    }
                }
                AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => {}
            }
        }

        updates
    }

    /// User 集約を更新する (別トランザクション)
    async fn update_user_aggregates(
        &self,
        account_id: &AccountId,
        user_updates: Vec<(UserId, UserUpdateAction)>,
    ) -> Result<(), ApplicationError> {
        for (user_id, action) in user_updates {
            // User 集約を読み込み
            let user_events = UserRepository::load_events(&self.user_repository, &user_id).await?;
            let user = User::from_events(user_events);

            // コマンドを実行
            let command = match action {
                UserUpdateAction::AddAccount => UserCommand::AddAccount {
                    user_id,
                    account_id: *account_id,
                },
                UserUpdateAction::RemoveAccount => UserCommand::RemoveAccount {
                    user_id,
                    account_id: *account_id,
                },
            };

            let new_events = match user.handle_command(command) {
                Ok(events) => events,
                Err(domain::UserError::AccountAlreadyAdded) => {
                    // 既に追加済み - スキップ
                    continue;
                }
                Err(domain::UserError::AccountNotFound) => {
                    // 既に削除済み - スキップ
                    continue;
                }
                Err(e) => {
                    return Err(ApplicationError::Repository(e.to_string()));
                }
            };

            // 新しいイベントを保存
            if !new_events.is_empty() {
                UserRepository::save_events(&self.user_repository, &user_id, new_events).await?;
            }
        }

        Ok(())
    }

    /// 月別サマリードキュメントのパス: `accounts/{account_id}/stats/monthly`
    fn monthly_summary_document_path(account_id: &AccountId) -> String {
        format!("accounts/{}/stats/monthly", account_id)
    }

    /// 日付文字列 ("YYYY-MM-DD") から月キー ("YYYY-MM") を取得する
    fn month_key_from_date(date: &str) -> String {
        // date は "YYYY-MM-DD" 形式
        date[..7].to_string()
    }

    /// 金額文字列を i64 にパースする
    fn parse_amount(amount: &str) -> i64 {
        amount.parse::<i64>().unwrap_or(0)
    }

    /// `accounts/*` への書き込みをトランザクション内で実行する
    async fn build_query_account_writes_in_tx(
        firestore: &Firestore,
        account_id: &AccountId,
        events: &[AccountEvent],
        aggregate: &Account,
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        // accounts/{account_id}/events/{event_id} へのイベント書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let document_path = Self::query_event_document_path(account_id, &event_id);
            let document_ref = firestore.doc(document_path)?;
            transaction.create(&document_ref, event)?;
        }

        // accounts/{account_id} へのアカウントドキュメント書き込み
        let document_path = Self::query_account_document_path(account_id);
        let document_ref = firestore.doc(document_path)?;
        let document_snapshot = transaction.get(&document_ref).await?;

        match document_snapshot.data::<QueryAccountDocumentData>() {
            None => {
                // 新規作成 (AccountCreated イベントがある場合のみ)
                if let Some(AccountEvent::AccountCreated { name, owners, .. }) = events.first() {
                    let document_data = QueryAccountDocumentData {
                        deleted_at: None,
                        id: account_id.to_string(),
                        name: name.clone(),
                        owners: owners.clone(),
                    };
                    transaction.create(&document_ref, &document_data)?;
                }
            }
            Some(result) => {
                let mut document_data: QueryAccountDocumentData = result?;

                for event in events {
                    match event {
                        AccountEvent::AccountUpdated { name, .. } => {
                            document_data.name = name.clone();
                        }
                        AccountEvent::AccountDeleted { common, .. } => {
                            document_data.deleted_at = Some(common.at.clone());
                        }
                        AccountEvent::OwnerAdded { owner, .. } => {
                            if !document_data.owners.contains(owner) {
                                document_data.owners.push(owner.clone());
                            }
                        }
                        AccountEvent::OwnerRemoved { owner, .. } => {
                            document_data.owners.retain(|o| o != owner);
                        }
                        AccountEvent::AccountCreated { .. }
                        | AccountEvent::CategoryAdded { .. }
                        | AccountEvent::CategoryDeleted { .. }
                        | AccountEvent::CategoryUpdated { .. }
                        | AccountEvent::TransactionAdded { .. }
                        | AccountEvent::TransactionDeleted { .. }
                        | AccountEvent::TransactionUpdated { .. } => {}
                    }
                }

                transaction.update(
                    &document_ref,
                    &document_data,
                    Precondition {
                        exists: Some(true),
                        last_update_time: None,
                    },
                )?;
            }
        }

        // 月別サマリーの更新
        Self::update_monthly_summary_in_tx(firestore, account_id, events, aggregate, transaction)
            .await?;

        // 取引クエリ用ドキュメントの更新
        Self::update_query_transactions_in_tx(firestore, account_id, events, transaction).await?;

        Ok(())
    }

    /// 取引クエリ用ドキュメント (`accounts/{account_id}/transactions/{transaction_id}`)
    /// をトランザクション内で更新する
    ///
    /// 参照系 (`TransactionProjection`) で event replay を回避するための read model。
    /// - `TransactionAdded`: ドキュメントを新規作成
    /// - `TransactionUpdated`: 既存ドキュメントから `created_at` を保持しつつ他フィールドを更新。
    ///   既存ドキュメントが無い場合 (backfill 未実施のレガシーデータ) は新規作成にフォールバック。
    /// - `TransactionDeleted`: ドキュメントを削除。既存が無ければ no-op。
    async fn update_query_transactions_in_tx(
        firestore: &Firestore,
        account_id: &AccountId,
        events: &[AccountEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        for event in events {
            match event {
                AccountEvent::TransactionAdded {
                    common,
                    props,
                    transaction_id,
                } => {
                    let transaction_id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id");
                    let document_path =
                        Self::query_transaction_document_path(account_id, &transaction_id);
                    let document_ref = firestore.doc(document_path)?;
                    let data = QueryAccountTransactionDocumentData {
                        account_id: account_id.to_string(),
                        amount: props.amount.clone(),
                        category_id: props.category_id.clone(),
                        comment: props.comment.clone(),
                        created_at: common.at.clone(),
                        date: props.date.clone(),
                        id: transaction_id.to_string(),
                        updated_at: common.at.clone(),
                    };
                    transaction.create(&document_ref, &data)?;
                }
                AccountEvent::TransactionUpdated {
                    common,
                    props,
                    transaction_id,
                } => {
                    let transaction_id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id");
                    let document_path =
                        Self::query_transaction_document_path(account_id, &transaction_id);
                    let document_ref = firestore.doc(document_path)?;
                    let document_snapshot = transaction.get(&document_ref).await?;

                    match document_snapshot.data::<QueryAccountTransactionDocumentData>() {
                        Some(result) => {
                            let existing = result?;
                            let data = QueryAccountTransactionDocumentData {
                                account_id: account_id.to_string(),
                                amount: props.amount.clone(),
                                category_id: props.category_id.clone(),
                                comment: props.comment.clone(),
                                created_at: existing.created_at,
                                date: props.date.clone(),
                                id: transaction_id.to_string(),
                                updated_at: common.at.clone(),
                            };
                            transaction.update(
                                &document_ref,
                                &data,
                                Precondition {
                                    exists: Some(true),
                                    last_update_time: None,
                                },
                            )?;
                        }
                        None => {
                            // backfill 未実施の transaction に対する update。
                            // created_at が不明なので updated_at と同値で新規作成する。
                            let data = QueryAccountTransactionDocumentData {
                                account_id: account_id.to_string(),
                                amount: props.amount.clone(),
                                category_id: props.category_id.clone(),
                                comment: props.comment.clone(),
                                created_at: common.at.clone(),
                                date: props.date.clone(),
                                id: transaction_id.to_string(),
                                updated_at: common.at.clone(),
                            };
                            transaction.create(&document_ref, &data)?;
                        }
                    }
                }
                AccountEvent::TransactionDeleted { transaction_id, .. } => {
                    let transaction_id: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id");
                    let document_path =
                        Self::query_transaction_document_path(account_id, &transaction_id);
                    let document_ref = firestore.doc(document_path)?;
                    let document_snapshot = transaction.get(&document_ref).await?;
                    if document_snapshot
                        .data::<QueryAccountTransactionDocumentData>()
                        .is_some()
                    {
                        transaction.delete(
                            &document_ref,
                            Precondition {
                                exists: Some(true),
                                last_update_time: None,
                            },
                        )?;
                    }
                    // 既存ドキュメントが無ければ no-op (backfill 未実施)
                }
                AccountEvent::AccountCreated { .. }
                | AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::OwnerAdded { .. }
                | AccountEvent::OwnerRemoved { .. } => {}
            }
        }
        Ok(())
    }

    /// 月別サマリードキュメントをトランザクション内で更新する
    async fn update_monthly_summary_in_tx(
        firestore: &Firestore,
        account_id: &AccountId,
        events: &[AccountEvent],
        aggregate: &Account,
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        // トランザクション関連のイベントがあるか確認
        let has_transaction_events = events.iter().any(|e| {
            matches!(
                e,
                AccountEvent::TransactionAdded { .. }
                    | AccountEvent::TransactionUpdated { .. }
                    | AccountEvent::TransactionDeleted { .. }
            )
        });
        if !has_transaction_events {
            return Ok(());
        }

        // 既存のサマリードキュメントを読み込む
        let document_path = Self::monthly_summary_document_path(account_id);
        let document_ref = firestore.doc(document_path)?;
        let document_snapshot = transaction.get(&document_ref).await?;

        let mut summary = match document_snapshot.data::<QueryAccountMonthlySummaryDocumentData>() {
            Some(result) => result?,
            None => QueryAccountMonthlySummaryDocumentData {
                id: account_id.to_string(),
                ..QueryAccountMonthlySummaryDocumentData::default()
            },
        };
        let is_update = document_snapshot
            .data::<QueryAccountMonthlySummaryDocumentData>()
            .is_some();

        // 集約からトランザクションマップを取得
        let transactions = match aggregate {
            Account::Active { transactions, .. } => transactions,
            Account::Empty => {
                // Empty の場合はトランザクションイベントは発生しないはず
                return Ok(());
            }
        };

        // イベントに基づいてサマリーを更新
        for event in events {
            match event {
                AccountEvent::TransactionAdded { props, .. } => {
                    let month_key = Self::month_key_from_date(&props.date);
                    let amount = Self::parse_amount(&props.amount);
                    // 追加: 元金額の符号で incomes/expenses を選び、その金額を加算
                    Self::apply_summary_delta(&mut summary, &month_key, amount, amount);
                }
                AccountEvent::TransactionUpdated {
                    transaction_id,
                    props,
                    ..
                } => {
                    // 旧トランザクションの金額・日付を集約から取得
                    let tid: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id");
                    if let Some(old_tx) = transactions.get(&tid) {
                        // 旧月の対応するバケットから減算
                        let old_month_key = Self::month_key_from_date(&old_tx.date);
                        let old_amount = Self::parse_amount(&old_tx.amount);
                        Self::apply_summary_delta(
                            &mut summary,
                            &old_month_key,
                            -old_amount,
                            old_amount,
                        );
                    }
                    // 新月の対応するバケットに加算
                    let new_month_key = Self::month_key_from_date(&props.date);
                    let new_amount = Self::parse_amount(&props.amount);
                    Self::apply_summary_delta(&mut summary, &new_month_key, new_amount, new_amount);
                }
                AccountEvent::TransactionDeleted { transaction_id, .. } => {
                    // 旧トランザクションの金額・日付を集約から取得
                    let tid: TransactionId = transaction_id
                        .parse()
                        .expect("Failed to parse transaction_id");
                    if let Some(old_tx) = transactions.get(&tid) {
                        let month_key = Self::month_key_from_date(&old_tx.date);
                        let amount = Self::parse_amount(&old_tx.amount);
                        // 削除: 元のバケットから減算
                        Self::apply_summary_delta(&mut summary, &month_key, -amount, amount);
                    }
                }
                AccountEvent::AccountCreated { .. }
                | AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::OwnerAdded { .. }
                | AccountEvent::OwnerRemoved { .. } => {}
            }
        }

        // サマリードキュメントを書き込む
        if is_update {
            transaction.update(
                &document_ref,
                &summary,
                Precondition {
                    exists: Some(true),
                    last_update_time: None,
                },
            )?;
        } else {
            transaction.create(&document_ref, &summary)?;
        }

        Ok(())
    }

    /// 月別サマリーに金額を加減算する
    ///
    /// - `delta`: 対応するバケットに加算する値（削除側なら負）
    /// - `classify_by`: バケット選択に使う元金額（正なら `incomes`、負なら `expenses`）
    fn apply_summary_delta(
        summary: &mut QueryAccountMonthlySummaryDocumentData,
        month_key: &str,
        delta: i64,
        classify_by: i64,
    ) {
        // incomes (>= 0) / expenses (< 0)
        let bucket = if classify_by >= 0 {
            &mut summary.incomes
        } else {
            &mut summary.expenses
        };
        let bucket_current = bucket
            .get(month_key)
            .map(|s| Self::parse_amount(s))
            .unwrap_or(0);
        bucket.insert(month_key.to_string(), (bucket_current + delta).to_string());
    }
}

/// Action to perform on a user document
#[derive(Clone, Debug)]
enum UserUpdateAction {
    AddAccount,
    RemoveAccount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use application::repository::AccountRepository;
    use bouzuya_firestore_client::FirestoreOptions;
    use domain::AccountEventCommonProps;
    use domain::AccountEventTransactionProps;
    use domain::TransactionId;

    /// テスト用のリポジトリを生成する
    async fn setup_repository() -> anyhow::Result<FirestoreAccountRepository> {
        let firestore = Firestore::new(FirestoreOptions {
            database_id: None,
            project_id: Some("demo-project".to_string()),
        })?;
        Ok(FirestoreAccountRepository::new(firestore))
    }

    /// テスト用の AccountCreated イベントを生成する
    fn account_created_event(account_id: &AccountId, event_id: &str, at: &str) -> AccountEvent {
        AccountEvent::AccountCreated {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: at.to_string(),
                id: event_id.to_string(),
                protocol_version: 3,
            },
            name: "テストアカウント".to_string(),
            // オーナーを空にしてユーザー集約の更新を回避する
            owners: vec![],
        }
    }

    /// テスト用の TransactionAdded イベントを生成する
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

    /// テスト用の TransactionUpdated イベントを生成する
    fn transaction_updated_event(
        account_id: &AccountId,
        event_id: &str,
        at: &str,
        transaction_id: &TransactionId,
        amount: &str,
        date: &str,
        comment: &str,
    ) -> AccountEvent {
        AccountEvent::TransactionUpdated {
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

    /// テスト用の TransactionDeleted イベントを生成する
    fn transaction_deleted_event(
        account_id: &AccountId,
        event_id: &str,
        at: &str,
        transaction_id: &TransactionId,
    ) -> AccountEvent {
        AccountEvent::TransactionDeleted {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: at.to_string(),
                id: event_id.to_string(),
                protocol_version: 3,
            },
            transaction_id: transaction_id.to_string(),
        }
    }

    /// 取引クエリ用ドキュメントを Firestore から読み出す
    async fn read_query_transaction_doc(
        repo: &FirestoreAccountRepository,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> anyhow::Result<Option<QueryAccountTransactionDocumentData>> {
        let path =
            FirestoreAccountRepository::query_transaction_document_path(account_id, transaction_id);
        let document_ref = repo.firestore.doc(path)?;
        let snapshot = document_ref.get().await?;
        Ok(snapshot
            .data::<QueryAccountTransactionDocumentData>()
            .transpose()?)
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_load_events_empty() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();

        let events = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(events, vec![]);
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_load_events_single_event() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let event = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");

        let aggregate = Account::Empty;
        AccountRepository::save_events(&repo, &account_id, vec![event.clone()], &aggregate).await?;
        let loaded = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(loaded, vec![event]);
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_load_events_sorted_by_at() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let event1 = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");
        let event2 = AccountEvent::AccountUpdated {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: "2024-01-02T00:00:00Z".to_string(),
                id: "evt-002".to_string(),
                protocol_version: 3,
            },
            name: "更新後アカウント".to_string(),
        };

        let aggregate = Account::Empty;
        AccountRepository::save_events(&repo, &account_id, vec![event1.clone()], &aggregate)
            .await?;
        let aggregate = Account::from_events(vec![event1.clone()]);
        AccountRepository::save_events(&repo, &account_id, vec![event2.clone()], &aggregate)
            .await?;

        let loaded = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], event1);
        assert_eq!(loaded[1], event2);
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_transaction_added_creates_query_doc() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let transaction_id = TransactionId::generate();

        // AccountCreated を保存して Active 状態にする
        let created = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");
        let aggregate = Account::Empty;
        AccountRepository::save_events(&repo, &account_id, vec![created.clone()], &aggregate)
            .await?;

        // TransactionAdded を保存
        let added = transaction_added_event(
            &account_id,
            "evt-002",
            "2024-01-02T10:00:00Z",
            &transaction_id,
            "-1000",
            "2024-01-02",
            "ランチ",
        );
        let aggregate = Account::from_events(vec![created.clone()]);
        AccountRepository::save_events(&repo, &account_id, vec![added.clone()], &aggregate).await?;

        // 取引クエリ用ドキュメントが生成されている
        let doc = read_query_transaction_doc(&repo, &account_id, &transaction_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("query transaction doc not found"))?;
        assert_eq!(doc.id, transaction_id.to_string());
        assert_eq!(doc.account_id, account_id.to_string());
        assert_eq!(doc.amount, "-1000");
        assert_eq!(doc.category_id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(doc.comment, "ランチ");
        assert_eq!(doc.date, "2024-01-02");
        assert_eq!(doc.created_at, "2024-01-02T10:00:00Z");
        assert_eq!(doc.updated_at, "2024-01-02T10:00:00Z");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_transaction_updated_preserves_created_at() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let transaction_id = TransactionId::generate();

        // AccountCreated + TransactionAdded を保存
        let created = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");
        AccountRepository::save_events(&repo, &account_id, vec![created.clone()], &Account::Empty)
            .await?;

        let added = transaction_added_event(
            &account_id,
            "evt-002",
            "2024-01-02T10:00:00Z",
            &transaction_id,
            "-1000",
            "2024-01-02",
            "初期",
        );
        let aggregate = Account::from_events(vec![created.clone()]);
        AccountRepository::save_events(&repo, &account_id, vec![added.clone()], &aggregate).await?;

        // TransactionUpdated を保存
        let updated = transaction_updated_event(
            &account_id,
            "evt-003",
            "2024-01-03T11:00:00Z",
            &transaction_id,
            "-2000",
            "2024-01-15",
            "更新後",
        );
        let aggregate = Account::from_events(vec![created, added]);
        AccountRepository::save_events(&repo, &account_id, vec![updated], &aggregate).await?;

        let doc = read_query_transaction_doc(&repo, &account_id, &transaction_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("query transaction doc not found"))?;
        assert_eq!(doc.amount, "-2000");
        assert_eq!(doc.date, "2024-01-15");
        assert_eq!(doc.comment, "更新後");
        // created_at は保持される
        assert_eq!(doc.created_at, "2024-01-02T10:00:00Z");
        // updated_at は新しい event の at に更新される
        assert_eq!(doc.updated_at, "2024-01-03T11:00:00Z");
        Ok(())
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn test_transaction_deleted_removes_query_doc() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let transaction_id = TransactionId::generate();

        // AccountCreated + TransactionAdded を保存
        let created = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");
        AccountRepository::save_events(&repo, &account_id, vec![created.clone()], &Account::Empty)
            .await?;

        let added = transaction_added_event(
            &account_id,
            "evt-002",
            "2024-01-02T10:00:00Z",
            &transaction_id,
            "-1000",
            "2024-01-02",
            "",
        );
        let aggregate = Account::from_events(vec![created.clone()]);
        AccountRepository::save_events(&repo, &account_id, vec![added.clone()], &aggregate).await?;

        // 削除前は存在する
        assert!(
            read_query_transaction_doc(&repo, &account_id, &transaction_id)
                .await?
                .is_some()
        );

        // TransactionDeleted を保存
        let deleted = transaction_deleted_event(
            &account_id,
            "evt-003",
            "2024-01-03T12:00:00Z",
            &transaction_id,
        );
        let aggregate = Account::from_events(vec![created, added]);
        AccountRepository::save_events(&repo, &account_id, vec![deleted], &aggregate).await?;

        // 削除後は無い
        assert!(
            read_query_transaction_doc(&repo, &account_id, &transaction_id)
                .await?
                .is_none()
        );
        Ok(())
    }
}
