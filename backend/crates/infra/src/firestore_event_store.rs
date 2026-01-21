use application::error::ApplicationError;
use application::repository::EventStoreRepository;
use async_trait::async_trait;
use domain::account::AccountEvent;
use domain::account::AccountId;
use firestore_client::FirestoreClient;
use firestore_client::path::{CollectionPath, DocumentPath};

/// Internal error type for FirestoreEventStore operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("firestore client: {0}")]
    FirestoreClient(#[from] firestore_client::Error),
}

impl From<E> for ApplicationError {
    fn from(e: E) -> Self {
        ApplicationError::Repository(e.to_string())
    }
}

/// Firestore-based event store implementation
#[derive(Clone)]
pub struct FirestoreEventStore {
    client: FirestoreClient,
}

impl FirestoreEventStore {
    /// Create a new FirestoreEventStore with the given client
    pub fn new(client: FirestoreClient) -> Self {
        Self { client }
    }

    /// Get the path to an event stream document: `aggregates/account/event_streams/{account_id}`
    fn event_stream_path(account_id: &AccountId) -> Result<DocumentPath, E> {
        let path_str = format!("aggregates/account/event_streams/{}", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to the events collection: `aggregates/account/event_streams/{account_id}/events`
    fn events_collection_path(account_id: &AccountId) -> Result<CollectionPath, E> {
        let path_str = format!("aggregates/account/event_streams/{}/events", account_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to an event document: `aggregates/account/event_streams/{account_id}/events/{eventId}`
    fn event_path(account_id: &AccountId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!(
            "aggregates/account/event_streams/{}/events/{}",
            account_id, event_id
        );
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a query event document: `accounts/{accountId}/events/{eventId}`
    fn query_event_path(account_id: &AccountId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!("accounts/{}/events/{}", account_id, event_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a user document: `users/{uid}`
    fn user_path(uid: &str) -> Result<DocumentPath, E> {
        let path_str = format!("users/{}", uid);
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

/// Event stream document schema
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EventStreamDocument {
    id: String,
    last_event_id: String,
    owners: Vec<String>,
    protocol_version: u32,
    updated_at: String,
}

/// User document schema
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UserDocument {
    id: String,
    account_ids: Vec<String>,
}

#[async_trait]
impl EventStoreRepository for FirestoreEventStore {
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

impl FirestoreEventStore {
    async fn load_events_impl(&self, account_id: &AccountId) -> Result<Vec<AccountEvent>, E> {
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

    async fn save_events_impl(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
    ) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        // Begin transaction
        let transaction = self.client.begin_transaction().await?;

        // Read current event stream document (for optimistic locking)
        let event_stream_path = Self::event_stream_path(account_id)?;
        let existing_stream = self
            .client
            .get_document_with_tx(event_stream_path.clone(), &transaction)
            .await?;

        // Get the last event to update the event stream
        let last_event = events.last().expect("events is non-empty");
        let last_event_id = Self::get_event_id(last_event);
        let last_event_at = Self::get_event_at(last_event);

        // Build writes for all events
        let mut writes = Vec::new();

        // Collect user updates (owner changes).
        // Since we use a HashMap keyed by owner UID, each owner will have at most one
        // UserUpdateAction. This is sufficient because within a single save_events call,
        // the final state of each owner's relationship to this account is what matters.
        let mut user_updates: std::collections::HashMap<String, UserUpdateAction> =
            std::collections::HashMap::new();

        for event in &events {
            let event_id = Self::get_event_id(event);

            // Write to accounts/{accountId}/events/{eventId}
            let event_path = Self::event_path(account_id, event_id)?;
            let event_value = self.client.serialize(event)?;
            writes.push(self.client.build_create_write(event_path, event_value));

            // Write to accountsForQuery/{accountId}/events/{eventId}
            let query_event_path = Self::query_event_path(account_id, event_id)?;
            let query_event_value = self.client.serialize(event)?;
            writes.push(
                self.client
                    .build_create_write(query_event_path, query_event_value),
            );

            // Track user updates for owner changes
            match event {
                AccountEvent::AccountCreated { owners, .. } => {
                    for owner in owners {
                        user_updates.insert(owner.clone(), UserUpdateAction::AddAccount);
                    }
                }
                AccountEvent::OwnerAdded { owner, .. } => {
                    user_updates.insert(owner.clone(), UserUpdateAction::AddAccount);
                }
                AccountEvent::OwnerRemoved { owner, .. } => {
                    user_updates.insert(owner.clone(), UserUpdateAction::RemoveAccount);
                }
                AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => {
                    // do nothing
                }
            }
        }

        // Build event stream document write based on whether this is a new or existing account
        match existing_stream {
            None => {
                // New account - get owners from the first event (should be AccountCreated)
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

                let event_stream = EventStreamDocument {
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
                // Existing account - update the event stream
                let mut event_stream: EventStreamDocument =
                    self.client.deserialize(existing_doc.fields)?;

                // Update owners based on events
                for event in &events {
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

        // Build user document writes
        for (uid, action) in user_updates {
            let user_path = Self::user_path(&uid)?;

            // Get existing user document
            let existing_user = self
                .client
                .get_document_with_tx(user_path.clone(), &transaction)
                .await?;

            let mut user_doc = match existing_user {
                Some(doc) => self.client.deserialize::<UserDocument>(doc.fields)?,
                None => UserDocument {
                    id: uid.clone(),
                    account_ids: vec![],
                },
            };

            let account_id_str = account_id.to_string();
            match action {
                UserUpdateAction::AddAccount => {
                    if !user_doc.account_ids.contains(&account_id_str) {
                        user_doc.account_ids.push(account_id_str);
                    }
                }
                UserUpdateAction::RemoveAccount => {
                    user_doc.account_ids.retain(|id| id != &account_id_str);
                }
            }

            let value = self.client.serialize(&user_doc)?;
            writes.push(self.client.build_set_write(user_path, value));
        }

        // Commit transaction
        self.client.commit(&transaction, writes).await?;

        Ok(())
    }
}

/// Action to perform on a user document
#[derive(Clone, Debug)]
enum UserUpdateAction {
    AddAccount,
    RemoveAccount,
}
