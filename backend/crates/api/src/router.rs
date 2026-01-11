use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;
use axum::Router;
use axum::routing::get;
use axum::routing::post;

use crate::handler::command;
use crate::handler::query;
use crate::state::AppState;

/// Create the API router with all routes
pub fn create_router<R, AP, CP, TP>(state: Arc<AppState<R, AP, CP, TP>>) -> Router
where
    R: EventStoreRepository + Clone + Send + Sync + 'static,
    AP: AccountProjection + Clone + Send + Sync + 'static,
    CP: CategoryProjection + Clone + Send + Sync + 'static,
    TP: TransactionProjection + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/accounts", get(query::list_accounts::<R, AP, CP, TP>))
        .route(
            "/accounts/{account_id}",
            get(query::get_account::<R, AP, CP, TP>),
        )
        .route(
            "/accounts/{account_id}/categories",
            get(query::list_categories::<R, AP, CP, TP>),
        )
        .route(
            "/accounts/{account_id}/export/json",
            get(query::export_transactions::<R, AP, CP, TP>),
        )
        .route(
            "/accounts/{account_id}/transactions",
            get(query::list_transactions::<R, AP, CP, TP>),
        )
        .route(
            "/commands/add_category",
            post(command::add_category::<R, AP, CP, TP>),
        )
        .route(
            "/commands/add_owner",
            post(command::add_owner::<R, AP, CP, TP>),
        )
        .route(
            "/commands/add_transaction",
            post(command::add_transaction::<R, AP, CP, TP>),
        )
        .route(
            "/commands/create_account",
            post(command::create_account::<R, AP, CP, TP>),
        )
        .route(
            "/commands/delete_account",
            post(command::delete_account::<R, AP, CP, TP>),
        )
        .route(
            "/commands/delete_category",
            post(command::delete_category::<R, AP, CP, TP>),
        )
        .route(
            "/commands/delete_transaction",
            post(command::delete_transaction::<R, AP, CP, TP>),
        )
        .route(
            "/commands/remove_owner",
            post(command::remove_owner::<R, AP, CP, TP>),
        )
        .route(
            "/commands/update_account",
            post(command::update_account::<R, AP, CP, TP>),
        )
        .route(
            "/commands/update_category",
            post(command::update_category::<R, AP, CP, TP>),
        )
        .route(
            "/commands/update_transaction",
            post(command::update_transaction::<R, AP, CP, TP>),
        )
        .with_state(state)
}
