mod credentials;
mod error;
mod extractor;
mod handler;
mod iam_signer;
mod router;
mod session_claims;
mod signer;
mod state;

pub use self::credentials::CredentialsError;
pub use self::credentials::ServiceAccountCredentials;
pub use self::iam_signer::IamSessionTokenCreateError;
pub use self::iam_signer::IamSessionTokenCreator;
pub use self::iam_signer::IamSessionTokenVerifier;
pub use self::iam_signer::IamSessionTokenVerifyError;
pub use self::session_claims::SessionTokenClaims;
pub use self::signer::CreateError;
pub use self::signer::Creator;
pub use self::signer::Verifier;
pub use self::signer::VerifyError;
pub use self::state::AppState;

use std::path::Path;

/// API サーバーを起動する
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `public_dir` - 静的ファイルを配信するディレクトリ（None の場合は配信しない）
pub async fn run(state: AppState, public_dir: Option<&Path>) {
    let app = router::create_router(state, public_dir);

    let addr = "0.0.0.0:3000";
    println!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
