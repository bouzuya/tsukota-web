use crate::repository::Repository;
use crate::schema::QueryUserDocumentData;
use crate::schema::UserEventStreamDocumentData;
use application::error::ApplicationError;
use application::repository::UserRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use domain::UserEvent;
use domain::UserId;

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

    /// Get the path to a user document: `users/{user_id}`
    fn query_user_document_path(user_id: &UserId) -> String {
        format!("users/{}", user_id)
    }
}

impl Repository for FirestoreUserRepository {
    type Event = UserEvent;
    type EventAt = String;
    type EventId = String;
    type EventStream = UserEventStreamDocumentData;
    type EventStreamId = UserId;

    fn event_stream_collection_path() -> String {
        "aggregates/user/event_streams".to_string()
    }

    fn firestore(&self) -> &Firestore {
        &self.firestore
    }

    fn get_event_at(event: &Self::Event) -> Self::EventAt {
        match event {
            UserEvent::AccountAdded { common, .. }
            | UserEvent::AccountRemoved { common, .. }
            | UserEvent::UserCreated { common, .. } => common.at.clone(),
        }
    }

    fn get_event_id(event: &Self::Event) -> Self::EventId {
        match event {
            UserEvent::AccountAdded { common, .. }
            | UserEvent::AccountRemoved { common, .. }
            | UserEvent::UserCreated { common, .. } => common.id.clone(),
        }
    }

    fn new_event_stream(
        event_stream_id: &Self::EventStreamId,
        events: &[Self::Event],
        _stored_event_stream: Option<Self::EventStream>,
    ) -> Self::EventStream {
        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);
        UserEventStreamDocumentData {
            id: event_stream_id.to_string(),
            updated_at: last_event_at,
        }
    }
}

#[async_trait]
impl UserRepository for FirestoreUserRepository {
    async fn load_events(&self, user_id: &UserId) -> Result<Vec<UserEvent>, ApplicationError> {
        Repository::load_events(self, user_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn save_events(
        &self,
        user_id: &UserId,
        events: Vec<UserEvent>,
    ) -> Result<(), ApplicationError> {
        let user_id_owned = *user_id;
        let events_clone = events.clone();
        let firestore = self.firestore.clone();
        Repository::save_events(
            self,
            *user_id,
            events,
            Box::new(move |transaction| {
                Box::pin(async move {
                    // users/* (クエリ用コレクション) への書き込み
                    let user_path = Self::query_user_document_path(&user_id_owned);
                    let document_ref = firestore.doc(user_path)?;

                    // UserCreated イベントがある場合は新規作成
                    let has_user_created = events_clone
                        .iter()
                        .any(|e| matches!(e, UserEvent::UserCreated { .. }));

                    if has_user_created {
                        // 新規ユーザー作成: account_ids は後続のイベントで更新
                        let mut account_ids = Vec::new();

                        // 同じバッチ内の AccountAdded イベントを処理
                        for event in &events_clone {
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
                            id: user_id_owned.to_string(),
                        };
                        transaction.create(&document_ref, &user_doc)?;
                    } else {
                        // 既存ユーザー: AccountAdded / AccountRemoved イベントを処理
                        let has_account_changes = events_clone.iter().any(|e| {
                            matches!(
                                e,
                                UserEvent::AccountAdded { .. } | UserEvent::AccountRemoved { .. }
                            )
                        });

                        if has_account_changes {
                            let document_snapshot = transaction.get(&document_ref).await?;

                            let mut user_doc =
                                match document_snapshot.data::<QueryUserDocumentData>() {
                                    None => QueryUserDocumentData {
                                        account_ids: vec![],
                                        id: user_id_owned.to_string(),
                                    },
                                    Some(result) => result?,
                                };

                            for event in &events_clone {
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
                })
            }),
        )
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
}
