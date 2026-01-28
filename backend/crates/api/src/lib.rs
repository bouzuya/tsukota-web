mod error;
mod extractor;
mod handler;
mod router;
mod session_token;
mod state;

pub use self::session_token::CreateError;
pub use self::session_token::Creator;
pub use self::session_token::CredentialsError;
pub use self::session_token::IamSessionTokenCreateError;
pub use self::session_token::IamSessionTokenCreator;
pub use self::session_token::IamSessionTokenVerifier;
pub use self::session_token::IamSessionTokenVerifyError;
pub use self::session_token::ServiceAccountCredentials;
pub use self::session_token::SessionTokenClaims;
pub use self::session_token::Verifier;
pub use self::session_token::VerifyError;
pub use self::state::AppState;

use std::path::Path;

/// API サーバーを起動する
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `port` - ポート番号
/// * `public_dir` - 静的ファイルを配信するディレクトリ
/// * `base_path` - ベースパス（例: "/api"）
pub async fn run(state: AppState, port: u16, public_dir: &Path, base_path: &str) {
    let app = router::create_router(state, public_dir, base_path);

    let addr = format!("0.0.0.0:{}", port);
    println!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
