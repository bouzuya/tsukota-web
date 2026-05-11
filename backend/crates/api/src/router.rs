use std::path::Path;

use axum::Router;
use axum::routing::get;
use axum::routing::post;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tower_http::trace::TraceLayer;

use crate::handler;
use crate::router::auth::AuthState;
use crate::router::auth::create_auth_router;
use crate::state::AppState;

pub(crate) mod auth;

/// API ルーターを作成する
///
/// 既存の `/me` /`accounts/...` /`commands/...` (AppState) と、
/// `/auth/{signin,signup,callback,signout}` (AuthState) を 1 つの `Router<()>` に統合する。
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `auth_state` - auth router 用ステート
/// * `public_dir` - 静的ファイルを配信するディレクトリ（None の場合は静的ファイルを配信しない）
/// * `base_path` - ベースパス（例: "/lab/tsukota"）
pub fn create_router(
    state: AppState,
    auth_state: AuthState,
    public_dir: Option<&Path>,
    base_path: &str,
) -> Router<()> {
    let api_router = Router::new()
        .route("/accounts", get(handler::list_accounts))
        .route("/accounts/{account_id}", get(handler::get_account))
        .route(
            "/accounts/{account_id}/categories",
            get(handler::list_categories),
        )
        .route(
            "/accounts/{account_id}/export/json",
            get(handler::export_transactions),
        )
        .route(
            "/accounts/{account_id}/stats/monthly",
            get(handler::get_monthly_summary),
        )
        .route(
            "/accounts/{account_id}/transactions",
            get(handler::list_transactions),
        )
        .route("/commands/add_category", post(handler::add_category))
        .route("/commands/add_owner", post(handler::add_owner))
        .route("/commands/add_transaction", post(handler::add_transaction))
        .route("/commands/create_account", post(handler::create_account))
        .route(
            "/commands/create_session_token",
            post(handler::create_session_token),
        )
        .route("/commands/delete_account", post(handler::delete_account))
        .route("/commands/delete_category", post(handler::delete_category))
        .route(
            "/commands/delete_transaction",
            post(handler::delete_transaction),
        )
        .route("/commands/remove_owner", post(handler::remove_owner))
        .route("/commands/update_account", post(handler::update_account))
        .route("/commands/update_category", post(handler::update_category))
        .route(
            "/commands/update_transaction",
            post(handler::update_transaction),
        )
        .route("/me", get(handler::get_me))
        .with_state(state);

    let auth_router = Router::new().nest("/auth", create_auth_router(auth_state));

    let app = api_router
        .merge(auth_router)
        .layer(TraceLayer::new_for_http());

    match public_dir {
        None => {
            // 静的ファイルを配信しない場合は API + auth ルーターのみ
            if base_path.is_empty() {
                app
            } else {
                Router::new().nest(base_path, app)
            }
        }
        Some(public_dir) => {
            if base_path.is_empty() {
                // ベースパスが空の場合はそのままルートに配置
                app.nest_service("/assets", ServeDir::new(public_dir.join("assets")))
                    .fallback_service(ServeFile::new(public_dir.join("index.html")))
            } else {
                // ベースパスがある場合はネストする
                let assets_path = format!("{}/assets", base_path);
                Router::new()
                    .nest(base_path, app)
                    .nest_service(&assets_path, ServeDir::new(public_dir.join("assets")))
                    .fallback_service(ServeFile::new(public_dir.join("index.html")))
            }
        }
    }
}
