mod credentials;
mod error;
mod extractor;
mod handler;
mod router;
mod signer;
mod state;

pub use credentials::CredentialsError;
pub use credentials::ServiceAccountCredentials;
pub use signer::SignError;
pub use signer::Signer;
pub use state::AppState;

/// API サーバーを起動する
pub async fn run(state: AppState) {
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
