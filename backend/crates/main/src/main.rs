use api::AppState;
use infra::InMemoryEventStore;
use infra::InMemoryProjection;

#[tokio::main]
async fn main() {
    // Initialize infrastructure components
    let event_store = InMemoryEventStore::new();
    let projection = InMemoryProjection::with_events(event_store.events());

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
