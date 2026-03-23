use crate::schema::DeviceEventStreamDocumentData;
use crate::schema::QueryDeviceDocumentData;
use application::error::ApplicationError;
use application::repository::DeviceRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use bouzuya_firestore_client::TransactionOptions;
use domain::DeviceEvent;
use domain::DeviceId;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreDeviceRepository operations
#[derive(Debug, thiserror::Error)]
enum E {
    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("event deserialize: {0}")]
    EventDeserialize(#[source] bouzuya_firestore_client::Error),

    #[error("event not found")]
    EventNotFound,

    #[error("get all event documents: {0}")]
    GetAllEventDocuments(#[source] bouzuya_firestore_client::Error),

    #[error("list event documents: {0}")]
    ListEventDocuments(#[source] bouzuya_firestore_client::Error),

    #[error("transaction: {0}")]
    Transaction(#[source] bouzuya_firestore_client::Error),
}

impl From<E> for ApplicationError {
    fn from(e: E) -> Self {
        ApplicationError::Repository(e.to_string())
    }
}

/// Firestore-based device repository implementation
#[derive(Clone)]
pub struct FirestoreDeviceRepository {
    firestore: Firestore,
}

impl FirestoreDeviceRepository {
    /// Create a new FirestoreDeviceRepository with the given firestore instance
    pub fn new(firestore: Firestore) -> Self {
        Self { firestore }
    }

    /// Get the path to a device event stream document: `aggregates/device/event_streams/{device_id}`
    fn event_stream_document_path(device_id: &DeviceId) -> Result<DocumentPath, E> {
        let path_str = format!("aggregates/device/event_streams/{}", device_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to the events collection: `aggregates/device/event_streams/{device_id}/events`
    fn event_collection_path(device_id: &DeviceId) -> Result<CollectionPath, E> {
        let path_str = format!("aggregates/device/event_streams/{}/events", device_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to an event document: `aggregates/device/event_streams/{device_id}/events/{event_id}`
    fn event_document_path(device_id: &DeviceId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!(
            "aggregates/device/event_streams/{}/events/{}",
            device_id, event_id
        );
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a device document: `devices/{device_id}`
    fn query_device_document_path(device_id: &DeviceId) -> Result<DocumentPath, E> {
        let path_str = format!("devices/{}", device_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Extract event ID from a DeviceEvent
    fn get_event_id(event: &DeviceEvent) -> &str {
        match event {
            DeviceEvent::DeviceCreated { common, .. } => &common.id,
        }
    }

    /// Extract the `at` timestamp from a DeviceEvent
    fn get_event_at(event: &DeviceEvent) -> &str {
        match event {
            DeviceEvent::DeviceCreated { common, .. } => &common.at,
        }
    }
}

#[async_trait]
impl DeviceRepository for FirestoreDeviceRepository {
    async fn load_events(
        &self,
        device_id: &DeviceId,
    ) -> Result<Vec<DeviceEvent>, ApplicationError> {
        self.load_events_impl(device_id).await.map_err(Into::into)
    }

    async fn save_events(
        &self,
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), ApplicationError> {
        self.save_events_impl(device_id, events)
            .await
            .map_err(Into::into)
    }
}

impl FirestoreDeviceRepository {
    async fn load_events_impl(&self, device_id: &DeviceId) -> Result<Vec<DeviceEvent>, E> {
        let collection_path = Self::event_collection_path(device_id)?;
        let collection_ref = self
            .firestore
            .collection(collection_path.to_string())
            .expect("invalid collection path");
        let document_refs = collection_ref
            .list_documents()
            .await
            .map_err(E::ListEventDocuments)?;

        let snapshots = self
            .firestore
            .get_all(document_refs)
            .await
            .map_err(E::GetAllEventDocuments)?;

        let mut all_events = Vec::new();
        for snapshot in snapshots {
            let event = snapshot
                .data::<DeviceEvent>()
                .ok_or(E::EventNotFound)?
                .map_err(E::EventDeserialize)?;
            all_events.push(event);
        }

        // Sort events by their `at` timestamp to ensure correct ordering
        all_events.sort_by(|a, b| Self::get_event_at(a).cmp(Self::get_event_at(b)));

        Ok(all_events)
    }

    async fn save_events_impl(
        &self,
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        let firestore = self.firestore.clone();
        let device_id_owned = device_id.clone();
        self.firestore
            .run_transaction(
                |transaction| {
                    Box::pin(async move {
                        // ========================================
                        // aggregates/device/* (イベントストア)
                        // ========================================
                        Self::build_aggregate_writes_in_tx(
                            &firestore,
                            &device_id_owned,
                            &events,
                            transaction,
                        )
                        .await?;

                        // ========================================
                        // devices/* (クエリ用コレクション)
                        // ========================================
                        Self::build_query_device_writes_in_tx(
                            &firestore,
                            &device_id_owned,
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

    /// `aggregates/device/*` への書き込みをトランザクション内で実行する
    async fn build_aggregate_writes_in_tx(
        firestore: &Firestore,
        device_id: &DeviceId,
        events: &[DeviceEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        // イベントストリームドキュメントの読み込み (排他制御のために get を使用)
        let event_stream_path = Self::event_stream_document_path(device_id)
            .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
        let document_ref = firestore.doc(event_stream_path.to_string())?;
        let document_snapshot = transaction.get(&document_ref).await?;

        // イベントドキュメントの書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let document_path = Self::event_document_path(device_id, event_id)
                .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
            let document_ref = firestore.doc(document_path.to_string())?;
            transaction.create(&document_ref, event)?;
        }

        // イベントストリームドキュメントの更新
        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);

        match document_snapshot.data::<DeviceEventStreamDocumentData>() {
            None => {
                let event_stream = DeviceEventStreamDocumentData {
                    id: device_id.to_string(),
                    updated_at: last_event_at.to_string(),
                };
                transaction.create(&document_ref, &event_stream)?;
            }
            Some(result) => {
                let mut event_stream: DeviceEventStreamDocumentData = result?;
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

    /// `devices/*` への書き込みをトランザクション内で実行する
    async fn build_query_device_writes_in_tx(
        firestore: &Firestore,
        device_id: &DeviceId,
        events: &[DeviceEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        for event in events {
            match event {
                DeviceEvent::DeviceCreated {
                    encrypted_secret,
                    user_id,
                    ..
                } => {
                    let device_doc = QueryDeviceDocumentData {
                        id: device_id.to_string(),
                        encrypted_secret: encrypted_secret.clone(),
                        uid: user_id.clone(),
                    };
                    let device_path = Self::query_device_document_path(device_id)
                        .map_err(|e| bouzuya_firestore_client::Error::custom(e))?;
                    let document_ref = firestore.doc(device_path.to_string())?;
                    transaction.create(&document_ref, &device_doc)?;
                }
            }
        }

        Ok(())
    }
}
