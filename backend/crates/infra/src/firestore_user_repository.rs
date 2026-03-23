use crate::schema::QueryUserDocumentData;
use crate::schema::UserEventStreamDocumentData;
use application::error::ApplicationError;
use application::repository::UserRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use bouzuya_firestore_client::TransactionOptions;
use domain::UserEvent;
use domain::UserId;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreUserRepository operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("event deserialize: {0}")]
    EventDeserialize(#[source] bouzuya_firestore_client::Error),

    #[error("event not found")]
    EventNotFound,

    #[error("get all event documents for user {0}")]
    GetAllEventDocuments(UserId, #[source] bouzuya_firestore_client::Error),

    #[error("list event documents for user {0}")]
    ListEventDocuments(UserId, #[source] bouzuya_firestore_client::Error),

    #[error("transaction: {0}")]
    Transaction(#[source] bouzuya_firestore_client::Error),
}

impl From<E> for ApplicationError {
    fn from(e: E) -> Self {
        ApplicationError::Repository(e.to_string())
    }
}

/// Firestore-based user repository implementation
#[derive(Clone)]
pub struct FirestoreUserRepository {
    firestore: Firestore,
}

impl FirestoreUserRepository {
    /// Create a new FirestoreUserRepository with the given firestore instance
    pub fn new(firestore: Firestore) -> Self {
        Self { firestore }
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
        let collection_ref = self
            .firestore
            .collection(collection_path.to_string())
            .expect("invalid collection path");
        let document_refs = collection_ref
            .list_documents()
            .await
            .map_err(|e| E::ListEventDocuments(user_id.clone(), e))?;

        let snapshots = self
            .firestore
            .get_all(document_refs)
            .await
            .map_err(|e| E::GetAllEventDocuments(user_id.clone(), e))?;

        let mut all_events = Vec::new();
        for snapshot in snapshots {
            let event = snapshot
                .data::<UserEvent>()
                .ok_or(E::EventNotFound)?
                .map_err(E::EventDeserialize)?;
            all_events.push(event);
        }

        // Sort events by their `at` timestamp to ensure correct ordering
        all_events.sort_by(|a, b| Self::get_event_at(a).cmp(Self::get_event_at(b)));

        Ok(all_events)
    }

    async fn save_events_impl(&self, user_id: &UserId, events: Vec<UserEvent>) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        let firestore = self.firestore.clone();
        let user_id_owned = user_id.clone();
        self.firestore
            .run_transaction(
                |transaction| {
                    Box::pin(async move {
                        // ========================================
                        // aggregates/user/* (イベントストア)
                        // ========================================
                        Self::build_aggregate_writes_in_tx(
                            &firestore,
                            &user_id_owned,
                            &events,
                            transaction,
                        )
                        .await?;

                        // ========================================
                        // users/* (クエリ用コレクション)
                        // ========================================
                        Self::build_query_user_writes_in_tx(
                            &firestore,
                            &user_id_owned,
                            &events,
                            transaction,
                        )
                        .await?;

                        Ok(())
                    })
                },
                TransactionOptions::default(),
            )
            .await
            .map_err(E::Transaction)?;

        Ok(())
    }

    /// `aggregates/user/*` への書き込みをトランザクション内で実行する
    async fn build_aggregate_writes_in_tx(
        firestore: &Firestore,
        user_id: &UserId,
        events: &[UserEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        // イベントドキュメントの書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let document_path = Self::event_document_path(user_id, event_id)
                .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
            let document_ref = firestore.doc(document_path.to_string())?;
            transaction.create(&document_ref, event)?;
        }

        // イベントストリームドキュメントの更新 (排他制御のために get を使用)
        let event_stream_path = Self::event_stream_document_path(user_id)
            .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
        let document_ref = firestore.doc(event_stream_path.to_string())?;
        let document_snapshot = transaction.get(&document_ref).await?;

        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);

        match document_snapshot.data::<UserEventStreamDocumentData>() {
            None => {
                let event_stream = UserEventStreamDocumentData {
                    id: user_id.to_string(),
                    updated_at: last_event_at.to_string(),
                };
                transaction.create(&document_ref, &event_stream)?;
            }
            Some(result) => {
                let mut event_stream: UserEventStreamDocumentData = result?;
                event_stream.updated_at = last_event_at.to_string();
                transaction.update(
                    &document_ref,
                    &event_stream,
                    Precondition {
                        exists: Some(true),
                        last_update_time: None,
                    },
                )?;
            }
        }

        Ok(())
    }

    /// `users/*` への書き込みをトランザクション内で実行する
    async fn build_query_user_writes_in_tx(
        firestore: &Firestore,
        user_id: &UserId,
        events: &[UserEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        let user_path = Self::query_user_document_path(user_id)
            .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
        let document_ref = firestore.doc(user_path.to_string())?;

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
            transaction.create(&document_ref, &user_doc)?;
        } else {
            // 既存ユーザー: AccountAdded / AccountRemoved イベントを処理
            let has_account_changes = events.iter().any(|e| {
                matches!(
                    e,
                    UserEvent::AccountAdded { .. } | UserEvent::AccountRemoved { .. }
                )
            });

            if has_account_changes {
                let document_snapshot = transaction.get(&document_ref).await?;

                let mut user_doc = match document_snapshot.data::<QueryUserDocumentData>() {
                    None => QueryUserDocumentData {
                        account_ids: vec![],
                        id: user_id.to_string(),
                    },
                    Some(result) => result?,
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

                transaction.update(
                    &document_ref,
                    &user_doc,
                    Precondition {
                        exists: Some(true),
                        last_update_time: None,
                    },
                )?;
            }
        }

        Ok(())
    }
}
