use std::path::Path;

use axum::Router;
use axum::routing::get;
use axum::routing::post;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

use crate::handler::command;
use crate::handler::query;
use crate::state::AppState;

/// API ルーターを作成する
///
/// # Arguments
///
/// * `state` - アプリケーションステート
/// * `public_dir` - 静的ファイルを配信するディレクトリ
pub fn create_router(state: AppState, public_dir: &Path) -> Router<()> {
    Router::new()
        .route("/accounts", get(query::list_accounts))
        .route("/accounts/{account_id}", get(query::get_account))
        .route(
            "/accounts/{account_id}/categories",
            get(query::list_categories),
        )
        .route(
            "/accounts/{account_id}/export/json",
            get(query::export_transactions),
        )
        .route(
            "/accounts/{account_id}/transactions",
            get(query::list_transactions),
        )
        .route("/commands/add_category", post(command::add_category))
        .route("/commands/add_owner", post(command::add_owner))
        .route("/commands/add_transaction", post(command::add_transaction))
        .route("/commands/create_account", post(command::create_account))
        .route(
            "/commands/create_session_token",
            post(command::create_session_token),
        )
        .route("/commands/delete_account", post(command::delete_account))
        .route("/commands/delete_category", post(command::delete_category))
        .route(
            "/commands/delete_transaction",
            post(command::delete_transaction),
        )
        .route("/commands/remove_owner", post(command::remove_owner))
        .route("/commands/update_account", post(command::update_account))
        .route("/commands/update_category", post(command::update_category))
        .route(
            "/commands/update_transaction",
            post(command::update_transaction),
        )
        .with_state(state)
        .nest_service("/assets", ServeDir::new(public_dir.join("assets")))
        .fallback_service(ServeFile::new(public_dir.join("index.html")))
}
