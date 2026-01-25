mod credentials;
mod error;
mod extractor;
mod handler;
mod router;
mod signer;
mod state;

pub use credentials::CredentialsError;
pub use credentials::ServiceAccountCredentials;
pub use signer::CreateError;
pub use signer::Creator;
pub use signer::Verifier;
pub use signer::VerifyError;
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
