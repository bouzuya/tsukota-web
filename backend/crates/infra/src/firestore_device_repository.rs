use application::error::ApplicationError;
use application::repository::DeviceRepository;
use async_trait::async_trait;
use domain::DeviceEvent;
use domain::DeviceId;
use firestore_client::FirestoreClient;
use firestore_client::path::CollectionPath;
use firestore_client::path::DocumentPath;

/// Internal error type for FirestoreDeviceRepository operations
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

/// Firestore-based device repository implementation
#[derive(Clone)]
pub struct FirestoreDeviceRepository {
    client: FirestoreClient,
}

impl FirestoreDeviceRepository {
    /// Create a new FirestoreDeviceRepository with the given client
    pub fn new(client: FirestoreClient) -> Self {
        Self { client }
    }

    /// Get the path to a device event stream document: `aggregates/device/event_streams/{device_id}`
    fn event_stream_path(device_id: &DeviceId) -> Result<DocumentPath, E> {
        let path_str = format!("aggregates/device/event_streams/{}", device_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to the events collection: `aggregates/device/event_streams/{device_id}/events`
    fn events_collection_path(device_id: &DeviceId) -> Result<CollectionPath, E> {
        let path_str = format!("aggregates/device/event_streams/{}/events", device_id);
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to an event document: `aggregates/device/event_streams/{device_id}/events/{event_id}`
    fn event_path(device_id: &DeviceId, event_id: &str) -> Result<DocumentPath, E> {
        let path_str = format!(
            "aggregates/device/event_streams/{}/events/{}",
            device_id, event_id
        );
        path_str.parse().map_err(|_| E::InvalidPath(path_str))
    }

    /// Get the path to a device document: `devices/{device_id}`
    fn device_path(device_id: &DeviceId) -> Result<DocumentPath, E> {
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

/// Device event stream document schema
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceEventStreamDocumentData {
    id: String,
    updated_at: String,
}

/// Device document schema
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceDocumentData {
    id: String,
    encrypted_secret: String,
    uid: String,
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
        let collection_path = Self::events_collection_path(device_id)?;

        let mut all_events = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let response = self
                .client
                .list_documents(collection_path.clone(), page_token)
                .await?;

            for doc in response.documents {
                let event: DeviceEvent = self.client.deserialize(doc.fields)?;
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
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), E> {
        if events.is_empty() {
            return Ok(());
        }

        // Begin transaction
        let transaction = self.client.begin_transaction().await?;

        // Read current event stream document (for optimistic locking)
        let event_stream_path = Self::event_stream_path(device_id)?;
        let existing_stream = self
            .client
            .get_document_with_tx(event_stream_path.clone(), &transaction)
            .await?;

        // Get the last event to update the event stream
        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);

        // Build writes for all events
        let mut writes = Vec::new();

        for event in &events {
            let event_id = Self::get_event_id(event);

            // Write to aggregates/device/event_streams/{device_id}/events/{event_id}
            let event_path = Self::event_path(device_id, event_id)?;
            let event_value = self.client.serialize(event)?;
            writes.push(self.client.build_create_write(event_path, event_value));

            // Write to devices/{device_id} for DeviceCreated event
            match event {
                DeviceEvent::DeviceCreated {
                    encrypted_secret,
                    user_id,
                    ..
                } => {
                    let device_doc = DeviceDocumentData {
                        id: device_id.to_string(),
                        encrypted_secret: encrypted_secret.clone(),
                        uid: user_id.clone(),
                    };
                    let device_path = Self::device_path(device_id)?;
                    let device_value = self.client.serialize(&device_doc)?;
                    writes.push(self.client.build_create_write(device_path, device_value));
                }
            }
        }

        // Build event stream document write based on whether this is a new or existing device
        match existing_stream {
            None => {
                // New device
                let event_stream = DeviceEventStreamDocumentData {
                    id: device_id.to_string(),
                    updated_at: last_event_at.to_string(),
                };

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_create_write(event_stream_path, value));
            }
            Some(existing_doc) => {
                // Existing device - update the event stream
                let mut event_stream: DeviceEventStreamDocumentData =
                    self.client.deserialize(existing_doc.fields)?;

                event_stream.updated_at = last_event_at.to_string();

                let value = self.client.serialize(&event_stream)?;
                writes.push(self.client.build_update_write(event_stream_path, value));
            }
        }

        // Commit transaction
        self.client.commit(&transaction, writes).await?;

        Ok(())
    }
}
