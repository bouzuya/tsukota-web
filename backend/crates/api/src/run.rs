use std::path::Path;

use tracing::info;

use crate::router;
use crate::state::AppState;

/// API サーバーを起動する
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `port` - ポート番号
/// * `public_dir` - 静的ファイルを配信するディレクトリ（None の場合は静的ファイルを配信しない）
/// * `base_path` - ベースパス（例: "/api"）
pub async fn run(state: AppState, port: u16, public_dir: Option<&Path>, base_path: &str) {
    let app = router::create_router(state, public_dir, base_path);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!(%addr, "サーバーを起動します");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
