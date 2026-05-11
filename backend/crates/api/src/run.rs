use std::path::Path;

use crate::router;
use crate::router::auth::AuthState;
use crate::state::AppState;

/// API サーバーを起動する
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `auth_state` - auth router 用ステート (OIDC client / SignIn・SignUp use case 等)
/// * `port` - ポート番号
/// * `public_dir` - 静的ファイルを配信するディレクトリ（None の場合は静的ファイルを配信しない）
/// * `base_path` - ベースパス（例: "/api"）
pub async fn run(
    state: AppState,
    auth_state: AuthState,
    port: u16,
    public_dir: Option<&Path>,
    base_path: &str,
) {
    let app = router::create_router(state, auth_state, public_dir, base_path);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!(%addr, "サーバーを起動します");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
