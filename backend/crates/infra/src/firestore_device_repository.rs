use crate::repository::Repository;
use crate::schema::DeviceEventStreamDocumentData;
use crate::schema::QueryDeviceDocumentData;
use application::error::ApplicationError;
use application::repository::DeviceRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use domain::DeviceEvent;
use domain::DeviceId;

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

    /// Get the path to a device document: `devices/{device_id}`
    fn query_device_document_path(device_id: &DeviceId) -> String {
        format!("devices/{}", device_id)
    }
}

impl Repository for FirestoreDeviceRepository {
    type Event = DeviceEvent;
    type EventAt = String;
    type EventId = String;
    type EventStream = DeviceEventStreamDocumentData;
    type EventStreamId = DeviceId;

    fn event_stream_collection_path() -> String {
        "aggregates/device/event_streams".to_string()
    }

    fn firestore(&self) -> &Firestore {
        &self.firestore
    }

    fn get_event_at(event: &Self::Event) -> Self::EventAt {
        match event {
            DeviceEvent::DeviceCreated { common, .. } => common.at.clone(),
        }
    }

    fn get_event_id(event: &Self::Event) -> Self::EventId {
        match event {
            DeviceEvent::DeviceCreated { common, .. } => common.id.clone(),
        }
    }

    fn new_event_stream(
        event_stream_id: &Self::EventStreamId,
        events: &[Self::Event],
        _stored_event_stream: Option<Self::EventStream>,
    ) -> Self::EventStream {
        let last_event = events.last().expect("events is non-empty");
        let last_event_at = Self::get_event_at(last_event);
        DeviceEventStreamDocumentData {
            id: event_stream_id.to_string(),
            updated_at: last_event_at,
        }
    }
}

#[async_trait]
impl DeviceRepository for FirestoreDeviceRepository {
    async fn load_events(
        &self,
        device_id: &DeviceId,
    ) -> Result<Vec<DeviceEvent>, ApplicationError> {
        Repository::load_events(self, device_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn save_events(
        &self,
        device_id: &DeviceId,
        events: Vec<DeviceEvent>,
    ) -> Result<(), ApplicationError> {
        let device_id_owned = *device_id;
        let events_clone = events.clone();
        let firestore = self.firestore.clone();
        Repository::save_events(
            self,
            *device_id,
            events,
            Box::new(move |transaction| {
                Box::pin(async move {
                    // devices/* (クエリ用コレクション) への書き込み
                    for event in &events_clone {
                        match event {
                            DeviceEvent::DeviceCreated {
                                encrypted_secret,
                                user_id,
                                ..
                            } => {
                                let device_doc = QueryDeviceDocumentData {
                                    id: device_id_owned.to_string(),
                                    encrypted_secret: encrypted_secret.clone(),
                                    uid: user_id.clone(),
                                };
                                let device_path =
                                    Self::query_device_document_path(&device_id_owned);
                                let document_ref = firestore.doc(device_path)?;
                                transaction.create(&document_ref, &device_doc)?;
                            }
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
