use crate::schema::QueryUserDocumentData;
use crate::schema::UserEventStreamDocumentData;
use application::error::ApplicationError;
use application::repository::UserRepository;
use async_trait::async_trait;
use domain::UserEvent;
use domain::UserId;
use firestore_client::FirestoreClient;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreUserRepository operations
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

/// Firestore-based user repository implementation
#[derive(Clone)]
pub struct FirestoreUserRepository {
    client: FirestoreClient,
}

impl FirestoreUserRepository {
    /// Create a new FirestoreUserRepository with the given client
    pub fn new(client: FirestoreClient) -> Self {
        Self { client }
    }

    /// Get the path to a user event stream document: `aggregates/user/event_streams/{user_id}`
    fn event_stream_document_path(user_id: &UserId) -> Result<DocumentPath, E> {
        let path_str = format!("aggregates/user/event_streams/{}", user_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to the events collection: `aggregates/user/event_streams/{user_id}/events`
    fn event_collection_path(user_id: &UserId) -> Result<CollectionPath, E> {
        let path_str = format!("aggregates/user/event_streams/{}/events", user_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to an event document: `aggregates/user/event_streams/{user_id}/events/{event_id}`
    fn event_document_path(user_id: &UserId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!(
            "aggregates/user/event_streams/{}/events/{}",
            user_id, event_id
        );
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a user document: `users/{user_id}`
    fn query_user_document_path(user_id: &UserId) -> Result<DocumentPath, E> {
        let path_str = format!("users/{}", user_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Extract event ID from a UserEvent
    fn get_event_id(event: &UserEvent) -> &str {
        match event {
            UserEvent::AccountAdded { common, .. }
            | UserEvent::AccountRemoved { common, .. }
            | UserEvent::UserCreated { common, .. } => &common.id,
        }
    }

    /// Extract the `at` timestamp from a UserEvent
    fn get_event_at(event: &UserEvent) -> &str {
        match event {
            UserEvent::AccountAdded { common, .. }
            | UserEvent::AccountRemoved { common, .. }
            | UserEvent::UserCreated { common, .. } => &common.at,
        }
    }
}

#[async_trait]
impl UserRepository for FirestoreUserRepository {
    async fn load_events(&self, user_id: &UserId) -> Result<Vec<UserEvent>, ApplicationError> {
        self.load_events_impl(user_id).await.map_err(Into::into)
    }

    async fn save_events(
        &self,
        user_id: &UserId,
        events: Vec<UserEvent>,
    ) -> Result<(), ApplicationError> {
        self.save_events_impl(user_id, events)
            .await
            .map_err(Into::into)
    }
}

impl FirestoreUserRepository {
    async fn load_events_impl(&self, user_id: &UserId) -> Result<Vec<UserEvent>, E> {
        let collection_path = Self::event_collection_path(user_id)?;

        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let response = self
                .client
                .list_documents(collection_path.clone(), page_token)
                .await?;

            for doc in response.documents {
                let event: UserEvent = self.client.deserialize(doc.fields)?;
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

    async fn save_events_impl(&self, user_id: &UserId, events: Vec<UserEvent>) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        let transaction = self.client.begin_transaction().await?;
        let mut writes = Vec::new();

        // ========================================
        // aggregates/user/* (イベントストア)
        // ========================================
        self.build_aggregate_writes(user_id, &events, &transaction, &mut writes)
            .await?;

        // ========================================
        // users/* (クエリ用コレクション)
        // ========================================
        self.build_query_user_writes(user_id, &events, &transaction, &mut writes)
            .await?;

        self.client.commit(&transaction, writes).await?;

        Ok(())
    }

    /// `aggregates/user/*` への書き込みを構築する
    async fn build_aggregate_writes(
        &self,
        user_id: &UserId,
        events: &[UserEvent],
        transaction: &firestore_client::FirestoreTransaction,
        writes: &mut Vec<firestore_client::google::firestore::v1::Write>,
    ) -> Result<(), E> {
        // イベントドキュメントの書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let event_path = Self::event_document_path(user_id, event_id)?;
            let event_value = self.client.serialize(event)?;
            writes.push(self.client.build_create_write(event_path, event_value));
        }

        // イベントストリームドキュメントの更新 (排他制御のために get_document_with_tx を使用)
        let event_stream_path = Self::event_stream_document_path(user_id)?;
        let existing_stream = self
            .client
            .get_document_with_tx(event_stream_path.clone(), transaction)
            .await?;

        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);

        match existing_stream {
            None => {
                let event_stream = UserEventStreamDocumentData {
                    id: user_id.to_string(),
                    updated_at: last_event_at.to_string(),
                };

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_create_write(event_stream_path, value));
            }
            Some(existing_doc) => {
                let mut event_stream: UserEventStreamDocumentData =
                    self.client.deserialize(existing_doc.fields)?;

                event_stream.updated_at = last_event_at.to_string();

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_update_write(event_stream_path, value));
            }
        }

        Ok(())
    }

    /// `users/*` への書き込みを構築する
    async fn build_query_user_writes(
        &self,
        user_id: &UserId,
        events: &[UserEvent],
        transaction: &firestore_client::FirestoreTransaction,
        writes: &mut Vec<firestore_client::google::firestore::v1::Write>,
    ) -> Result<(), E> {
        let user_path = Self::query_user_document_path(user_id)?;

        // UserCreated イベントがある場合は新規作成
        let has_user_created = events
            .iter()
            .any(|e| matches!(e, UserEvent::UserCreated { .. }));

        if has_user_created {
            // 新規ユーザー作成: account_ids は後続のイベントで更新
            let mut account_ids = Vec::new();

            // 同じバッチ内の AccountAdded イベントを処理
            for event in events {
                match event {
                    UserEvent::AccountAdded { account_id, .. } => {
                        if !account_ids.contains(account_id) {
                            account_ids.push(account_id.clone());
                        }
                    }
                    UserEvent::AccountRemoved { account_id, .. } => {
                        account_ids.retain(|id| id != account_id);
                    }
                    UserEvent::UserCreated { .. } => {}
                }
            }

            let user_doc = QueryUserDocumentData {
                account_ids,
                id: user_id.to_string(),
            };
            let user_value = self.client.serialize(&user_doc)?;
            writes.push(self.client.build_create_write(user_path, user_value));
        } else {
            // 既存ユーザー: AccountAdded / AccountRemoved イベントを処理
            let has_account_changes = events.iter().any(|e| {
                matches!(
                    e,
                    UserEvent::AccountAdded { .. } | UserEvent::AccountRemoved { .. }
                )
            });

            if has_account_changes {
                let existing_user = self
                    .client
                    .get_document_with_tx(user_path.clone(), transaction)
                    .await?;

                let mut user_doc = match existing_user {
                    Some(doc) => self.client.deserialize::<QueryUserDocumentData>(doc.fields)?,
                    None => QueryUserDocumentData {
                        account_ids: vec![],
                        id: user_id.to_string(),
                    },
                };

                for event in events {
                    match event {
                        UserEvent::AccountAdded { account_id, .. } => {
                            if !user_doc.account_ids.contains(account_id) {
                                user_doc.account_ids.push(account_id.clone());
                            }
                        }
                        UserEvent::AccountRemoved { account_id, .. } => {
                            user_doc.account_ids.retain(|id| id != account_id);
                        }
                        UserEvent::UserCreated { .. } => {}
                    }
                }

                let user_value = self.client.serialize(&user_doc)?;
                writes.push(self.client.build_update_write(user_path, user_value));
            }
        }

        Ok(())
    }
}
