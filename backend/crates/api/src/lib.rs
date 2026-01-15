mod create_custom_token;
mod error;
mod extractor;
mod handler;
mod router;
mod state;

pub use create_custom_token::{create_custom_token, CreateCustomTokenError};

use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;

pub use state::AppState;

/// Run the API server with the given state
pub async fn run<R, AP, CP, TP>(state: Arc<AppState<R, AP, CP, TP>>)
where
    R: EventStoreRepository + Clone + Send + Sync + 'static,
    AP: AccountProjection + Clone + Send + Sync + 'static,
    CP: CategoryProjection + Clone + Send + Sync + 'static,
    TP: TransactionProjection + Clone + Send + Sync + 'static,
{
    let app = router::create_router(state);

    let addr = "0.0.0.0:3000";
    println!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
