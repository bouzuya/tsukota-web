use crate::FirestoreUserRepository;
use crate::schema::AccountEventStreamDocumentData;
use crate::schema::QueryAccountDocumentData;
use application::error::ApplicationError;
use application::repository::AccountRepository;
use application::repository::UserRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use domain::AccountEvent;
use domain::AccountId;
use domain::User;
use domain::UserCommand;
use domain::UserId;
use firestore_client::FirestoreClient;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreAccountRepository operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("event deserialize for account {0}")]
    EventDeserialize(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("event not found for account {0}")]
    EventNotFound(AccountId),

    #[error("get event document for account {0}")]
    GetEventDocument(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("list event documents for account {0}")]
    ListEventDocuments(AccountId, #[source] bouzuya_firestore_client::Error),

    #[error("firestore client: {0}")]
    FirestoreClient(#[from] firestore_client::FirestoreClientError),

    #[error("user: {0}")]
    User(String),
}

impl From<E> for ApplicationError {
    fn from(e: E) -> Self {
        ApplicationError::Repository(e.to_string())
    }
}

/// Firestore-based event store implementation
#[derive(Clone)]
pub struct FirestoreAccountRepository {
    client: FirestoreClient,
    firestore: Firestore,
    user_repository: FirestoreUserRepository,
}

impl FirestoreAccountRepository {
    /// Create a new FirestoreAccountRepository with the given client
    pub fn new(client: FirestoreClient, firestore: Firestore) -> Self {
        let user_repository = FirestoreUserRepository::new(client.clone());
        Self {
            client,
            firestore,
            user_repository,
        }
    }

    /// Get the path to an event stream document: `aggregates/account/event_streams/{account_id}`
    fn event_stream_document_path(account_id: &AccountId) -> Result<DocumentPath, E> {
        let path_str = format!("aggregates/account/event_streams/{}", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to the events collection: `aggregates/account/event_streams/{account_id}/events`
    fn event_collection_path(account_id: &AccountId) -> Result<CollectionPath, E> {
        let path_str = format!("aggregates/account/event_streams/{}/events", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to an event document: `aggregates/account/event_streams/{account_id}/events/{eventId}`
    fn event_document_path(account_id: &AccountId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!(
            "aggregates/account/event_streams/{}/events/{}",
            account_id, event_id
        );
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a query account document: `accounts/{account_id}`
    fn query_account_document_path(account_id: &AccountId) -> Result<DocumentPath, E> {
        let path_str = format!("accounts/{}", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a query event document: `accounts/{account_id}/events/{event_id}`
    fn query_event_document_path(
        account_id: &AccountId,
        event_id: &str,
    ) -> Result<DocumentPath, E> {
        let path_str = format!("accounts/{}/events/{}", account_id, event_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Extract event ID from an AccountEvent
    fn get_event_id(event: &AccountEvent) -> &str {
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
            | AccountEvent::TransactionUpdated { common, .. } => &common.id,
        }
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
}

#[async_trait]
impl AccountRepository for FirestoreAccountRepository {
    async fn load_events(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<AccountEvent>, ApplicationError> {
        self.load_events_impl(account_id).await.map_err(Into::into)
    }

    async fn save_events(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
    ) -> Result<(), ApplicationError> {
        self.save_events_impl(account_id, events)
            .await
            .map_err(Into::into)
    }
}

impl FirestoreAccountRepository {
    async fn load_events_impl(&self, account_id: &AccountId) -> Result<Vec<AccountEvent>, E> {
        let collection_path = Self::event_collection_path(account_id)?;
        let collection_ref = self
            .firestore
            .collection(collection_path.to_string())
            .expect("invalid collection path");
        let document_refs = collection_ref
            .list_documents()
            .await
            .map_err(|e| E::ListEventDocuments(account_id.clone(), e))?;

        let snapshots = futures::future::join_all(
            document_refs
                .into_iter()
                .map(|it| async move { it.get().await }),
        )
        .await;

        let mut all_events = Vec::new();
        for snapshot in snapshots {
            let snapshot = snapshot.map_err(|e| E::GetEventDocument(account_id.clone(), e))?;
            let event = snapshot
                .data::<AccountEvent>()
                .ok_or_else(|| E::EventNotFound(account_id.clone()))?
                .map_err(|e| E::EventDeserialize(account_id.clone(), e))?;
            all_events.push(event);
        }

        // Sort events by their `at` timestamp to ensure correct ordering
        all_events.sort_by(|a, b| Self::get_event_at(a).cmp(Self::get_event_at(b)));

        Ok(all_events)
    }

    async fn save_events_impl(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
    ) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        // オーナー変更を収集 (User 集約の更新に使用)
        let user_updates = self.collect_user_updates(&events);

        let transaction = self.client.begin_transaction().await?;
        let mut writes = Vec::new();

        // ========================================
        // aggregates/account/* (イベントストア)
        // ========================================
        self.build_aggregate_writes(account_id, &events, &transaction, &mut writes)
            .await?;

        // ========================================
        // accounts/* (クエリ用コレクション)
        // ========================================
        self.build_query_account_writes(account_id, &events, &transaction, &mut writes)
            .await?;

        self.client.commit(&transaction, writes).await?;

        // ========================================
        // User 集約の更新 (別トランザクション)
        // ========================================
        self.update_user_aggregates(account_id, user_updates)
            .await?;

        Ok(())
    }

    /// AccountEvent からオーナー変更を収集する
    fn collect_user_updates(&self, events: &[AccountEvent]) -> Vec<(UserId, UserUpdateAction)> {
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
    ) -> Result<(), E> {
        for (user_id, action) in user_updates {
            // User 集約を読み込み
            let user_events = self
                .user_repository
                .load_events(&user_id)
                .await
                .map_err(|e| E::User(e.to_string()))?;
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
                    return Err(E::User(e.to_string()));
                }
            };

            // 新しいイベントを保存
            if !new_events.is_empty() {
                self.user_repository
                    .save_events(&user_id, new_events)
                    .await
                    .map_err(|e| E::User(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// `aggregates/account/*` への書き込みを構築する
    async fn build_aggregate_writes(
        &self,
        account_id: &AccountId,
        events: &[AccountEvent],
        transaction: &firestore_client::FirestoreTransaction,
        writes: &mut Vec<firestore_client::google::firestore::v1::Write>,
    ) -> Result<(), E> {
        // イベントドキュメントの書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let event_path = Self::event_document_path(account_id, event_id)?;
            let event_value = self.client.serialize(event)?;
            writes.push(self.client.build_create_write(event_path, event_value));
        }

        // イベントストリームドキュメントの更新 (排他制御のために get_document_with_tx を使用)
        let event_stream_path = Self::event_stream_document_path(account_id)?;
        let existing_stream = self
            .client
            .get_document_with_tx(event_stream_path.clone(), transaction)
            .await?;

        let last_event = events.last().expect("events is non-empty");
        let last_event_id = Self::get_event_id(last_event);
        let last_event_at = Self::get_event_at(last_event);

        match existing_stream {
            None => {
                let owners = match &events[0] {
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
                };

                let event_stream = AccountEventStreamDocumentData {
                    id: account_id.to_string(),
                    last_event_id: last_event_id.to_string(),
                    owners,
                    protocol_version: domain::Account::PROTOCOL_VERSION,
                    updated_at: last_event_at.to_string(),
                };

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_create_write(event_stream_path, value));
            }
            Some(existing_doc) => {
                let mut event_stream: AccountEventStreamDocumentData =
                    self.client.deserialize(existing_doc.fields)?;

                for event in events {
                    match event {
                        AccountEvent::OwnerAdded { owner, .. } => {
                            if !event_stream.owners.contains(owner) {
                                event_stream.owners.push(owner.clone());
                            }
                        }
                        AccountEvent::OwnerRemoved { owner, .. } => {
                            event_stream.owners.retain(|o| o != owner);
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

                event_stream.last_event_id = last_event_id.to_string();
                event_stream.updated_at = last_event_at.to_string();

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_update_write(event_stream_path, value));
            }
        }

        Ok(())
    }

    /// `accounts/*` への書き込みを構築する
    async fn build_query_account_writes(
        &self,
        account_id: &AccountId,
        events: &[AccountEvent],
        transaction: &firestore_client::FirestoreTransaction,
        writes: &mut Vec<firestore_client::google::firestore::v1::Write>,
    ) -> Result<(), E> {
        // accounts/{account_id}/events/{event_id} へのイベント書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let query_event_path = Self::query_event_document_path(account_id, event_id)?;
            let query_event_value = self.client.serialize(event)?;
            writes.push(
                self.client
                    .build_create_write(query_event_path, query_event_value),
            );
        }

        // accounts/{account_id} へのアカウントドキュメント書き込み
        let query_account_path = Self::query_account_document_path(account_id)?;
        let existing_account = self
            .client
            .get_document_with_tx(query_account_path.clone(), transaction)
            .await?;

        match existing_account {
            None => {
                // 新規作成 (AccountCreated イベントがある場合のみ)
                if let Some(AccountEvent::AccountCreated { name, owners, .. }) = events.first() {
                    let account_doc = QueryAccountDocumentData {
                        deleted_at: None,
                        id: account_id.to_string(),
                        name: name.clone(),
                        owners: owners.clone(),
                    };
                    let value = self.client.serialize(&account_doc)?;
                    writes.push(self.client.build_create_write(query_account_path, value));
                }
            }
            Some(existing_doc) => {
                let mut account_doc: QueryAccountDocumentData =
                    self.client.deserialize(existing_doc.fields)?;

                for event in events {
                    match event {
                        AccountEvent::AccountUpdated { name, .. } => {
                            account_doc.name = name.clone();
                        }
                        AccountEvent::AccountDeleted { common, .. } => {
                            account_doc.deleted_at = Some(common.at.clone());
                        }
                        AccountEvent::OwnerAdded { owner, .. } => {
                            if !account_doc.owners.contains(owner) {
                                account_doc.owners.push(owner.clone());
                            }
                        }
                        AccountEvent::OwnerRemoved { owner, .. } => {
                            account_doc.owners.retain(|o| o != owner);
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

                let value = self.client.serialize(&account_doc)?;
                writes.push(self.client.build_update_write(query_account_path, value));
            }
        }

        Ok(())
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

    /// テスト用のリポジトリを生成する
    async fn setup_repository() -> anyhow::Result<FirestoreAccountRepository> {
        let client = FirestoreClient::connect_with_emulator().await?;
        let firestore = Firestore::new(FirestoreOptions {
            project_id: Some("demo-project".to_string()),
        })?;
        Ok(FirestoreAccountRepository::new(client, firestore))
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

    #[tokio::test]
    async fn test_load_events_empty() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();

        let events = repo.load_events(&account_id).await?;

        assert_eq!(events, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_events_single_event() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let event = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");

        repo.save_events(&account_id, vec![event.clone()]).await?;
        let loaded = repo.load_events(&account_id).await?;

        assert_eq!(loaded, vec![event]);
        Ok(())
    }

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

        repo.save_events(&account_id, vec![event1.clone()]).await?;
        repo.save_events(&account_id, vec![event2.clone()]).await?;

        let loaded = repo.load_events(&account_id).await?;

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], event1);
        assert_eq!(loaded[1], event2);
        Ok(())
    }
}
