use api::AppState;
// use infra::DatabaseName;
use infra::FirestoreClient;
use infra::FirestoreEventStore;
use infra::FirestoreProjection;
// use infra::InMemoryEventStore;
// use infra::InMemoryProjection;

#[tokio::main]
async fn main() {
    // Initialize infrastructure components
    // let event_store = InMemoryEventStore::new();
    // let projection = InMemoryProjection::with_events(event_store.events());
    let client = FirestoreClient::connect_with_emulator().await.unwrap();
    let event_store = FirestoreEventStore::new(client.clone());
    let projection = FirestoreProjection::new(client);

    // Create application state with DI
    let state = AppState::new(
        event_store,
        projection.clone(),
        projection.clone(),
        projection,
    );

    // Run the server
    api::run(state).await;
}
