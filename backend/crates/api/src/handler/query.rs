use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;
use application::request::ExportTransactionsRequest;
use application::request::GetAccountRequest;
use application::request::ListAccountsRequest;
use application::request::ListCategoriesRequest;
use application::request::ListTransactionsRequest;
use application::view::AccountView;
use application::view::CategoryView;
use application::view::PaginatedList;
use application::view::TransactionView;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractor::AuthUser;
use crate::state::AppState;

// Account queries

pub async fn list_accounts<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<AccountView>>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let request = ListAccountsRequest {};
    let accounts = state.list_accounts.execute(&user_id, request).await?;
    Ok(Json(accounts))
}

pub async fn get_account<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
) -> Result<Json<AccountView>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let request = GetAccountRequest { account_id };
    let account = state.get_account.execute(&user_id, request).await?;
    Ok(Json(account))
}

// Category queries

pub async fn list_categories<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
) -> Result<Json<Vec<CategoryView>>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let request = ListCategoriesRequest { account_id };
    let categories = state.list_categories.execute(&user_id, request).await?;
    Ok(Json(categories))
}

// Transaction queries

#[derive(Debug, Deserialize)]
pub struct ListTransactionsParams {
    pub after: Option<String>,
}

const DEFAULT_PAGE_SIZE: usize = 20;

pub async fn list_transactions<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
    Query(params): Query<ListTransactionsParams>,
) -> Result<Json<PaginatedList<TransactionView>>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let request = ListTransactionsRequest {
        account_id,
        cursor: params.after,
        limit: DEFAULT_PAGE_SIZE,
    };
    let list = state.list_transactions.execute(&user_id, request).await?;
    Ok(Json(list))
}

// Export queries

#[derive(Debug, Deserialize)]
pub struct ExportTransactionsParams {
    pub year: i32,
    pub month: u32,
}

pub async fn export_transactions<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
    Query(params): Query<ExportTransactionsParams>,
) -> Result<Json<Vec<TransactionView>>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let request = ExportTransactionsRequest {
        account_id,
        year: params.year,
        month: params.month,
    };
    let transactions = state.export_transactions.execute(&user_id, request).await?;
    Ok(Json(transactions))
}
