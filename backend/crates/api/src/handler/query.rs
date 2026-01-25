use application::request::ExportTransactionsRequest;
use application::request::GetAccountRequest;
use application::request::ListAccountsRequest;
use application::request::ListCategoriesRequest;
use application::request::ListTransactionsRequest;
use application::response::ExportTransactionsResponse;
use application::response::GetAccountResponse;
use application::response::ListAccountsResponse;
use application::response::ListCategoriesResponse;
use application::response::ListTransactionsResponse;
use application::use_case::ExportTransactionsUseCase;
use application::use_case::GetAccountUseCase;
use application::use_case::ListAccountsUseCase;
use application::use_case::ListCategoriesUseCase;
use application::use_case::ListTransactionsUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractor::AuthUser;

// Account queries

pub async fn list_accounts(
    State(use_case): State<ListAccountsUseCase>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<ListAccountsResponse>, ApiError> {
    let request = ListAccountsRequest {};
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}

pub async fn get_account(
    State(use_case): State<GetAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
) -> Result<Json<GetAccountResponse>, ApiError> {
    let request = GetAccountRequest { account_id };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}

// Category queries

pub async fn list_categories(
    State(use_case): State<ListCategoriesUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
) -> Result<Json<ListCategoriesResponse>, ApiError> {
    let request = ListCategoriesRequest { account_id };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}

// Transaction queries

#[derive(Debug, Deserialize)]
pub struct ListTransactionsParams {
    pub after: Option<String>,
}

const DEFAULT_PAGE_SIZE: usize = 20;

pub async fn list_transactions(
    State(use_case): State<ListTransactionsUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
    Query(params): Query<ListTransactionsParams>,
) -> Result<Json<ListTransactionsResponse>, ApiError> {
    let request = ListTransactionsRequest {
        account_id,
        cursor: params.after,
        limit: DEFAULT_PAGE_SIZE,
    };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}

// Export queries

#[derive(Debug, Deserialize)]
pub struct ExportTransactionsParams {
    pub year: i32,
    pub month: u32,
}

pub async fn export_transactions(
    State(use_case): State<ExportTransactionsUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
    Query(params): Query<ExportTransactionsParams>,
) -> Result<Json<ExportTransactionsResponse>, ApiError> {
    let request = ExportTransactionsRequest {
        account_id,
        year: params.year,
        month: params.month,
    };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
