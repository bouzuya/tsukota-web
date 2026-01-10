use std::sync::Arc;

use application::error::ApplicationError;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;
use application::view::AccountView;
use application::view::CategoryView;
use application::view::TransactionList;
use application::view::TransactionView;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use domain::account::AccountId;
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
    let accounts = state.list_accounts.execute(&user_id).await?;
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
    let account_id: AccountId = account_id
        .parse()
        .map_err(|_| ApiError(ApplicationError::InvalidRequest("Invalid account ID".into())))?;
    let account = state.get_account.execute(&account_id, &user_id).await?;
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
    let account_id: AccountId = account_id
        .parse()
        .map_err(|_| ApiError(ApplicationError::InvalidRequest("Invalid account ID".into())))?;
    let categories = state
        .list_categories
        .execute(&account_id, &user_id)
        .await?;
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
) -> Result<Json<TransactionList>, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let account_id: AccountId = account_id
        .parse()
        .map_err(|_| ApiError(ApplicationError::InvalidRequest("Invalid account ID".into())))?;
    let list = state
        .list_transactions
        .execute(&account_id, &user_id, params.after, DEFAULT_PAGE_SIZE)
        .await?;
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
    let account_id: AccountId = account_id
        .parse()
        .map_err(|_| ApiError(ApplicationError::InvalidRequest("Invalid account ID".into())))?;
    let transactions = state
        .export_transactions
        .execute(&account_id, &user_id, params.year, params.month)
        .await?;
    Ok(Json(transactions))
}
