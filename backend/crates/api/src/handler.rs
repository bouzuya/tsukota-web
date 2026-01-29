use application::request::AddCategoryRequest;
use application::request::AddOwnerRequest;
use application::request::AddTransactionRequest;
use application::request::CreateAccountRequest;
use application::request::CreateSessionTokenRequest;
use application::request::DeleteAccountRequest;
use application::request::DeleteCategoryRequest;
use application::request::DeleteTransactionRequest;
use application::request::ExportTransactionsRequest;
use application::request::GetAccountRequest;
use application::request::ListAccountsRequest;
use application::request::ListCategoriesRequest;
use application::request::ListTransactionsRequest;
use application::request::RemoveOwnerRequest;
use application::request::UpdateAccountRequest;
use application::request::UpdateCategoryRequest;
use application::request::UpdateTransactionRequest;
use application::response::AddCategoryResponse;
use application::response::AddTransactionResponse;
use application::response::CreateAccountResponse;
use application::response::CreateSessionTokenResponse;
use application::response::ExportTransactionsResponse;
use application::response::GetAccountResponse;
use application::response::ListAccountsResponse;
use application::response::ListCategoriesResponse;
use application::response::ListTransactionsResponse;
use application::use_case::AddCategoryUseCase;
use application::use_case::AddOwnerUseCase;
use application::use_case::AddTransactionUseCase;
use application::use_case::CreateAccountUseCase;
use application::use_case::CreateSessionTokenUseCase;
use application::use_case::DeleteAccountUseCase;
use application::use_case::DeleteCategoryUseCase;
use application::use_case::DeleteTransactionUseCase;
use application::use_case::ExportTransactionsUseCase;
use application::use_case::GetAccountUseCase;
use application::use_case::ListAccountsUseCase;
use application::use_case::ListCategoriesUseCase;
use application::use_case::ListTransactionsUseCase;
use application::use_case::RemoveOwnerUseCase;
use application::use_case::UpdateAccountUseCase;
use application::use_case::UpdateCategoryUseCase;
use application::use_case::UpdateTransactionUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractor::AuthUser;

const DEFAULT_PAGE_SIZE: usize = 20;

#[derive(Debug, Deserialize)]
pub struct ExportTransactionsParams {
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsParams {
    pub after: Option<String>,
}

// Account commands

pub async fn create_account(
    State(use_case): State<CreateAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_account(
    State(use_case): State<UpdateAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_account(
    State(use_case): State<DeleteAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_owner(
    State(use_case): State<AddOwnerUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddOwnerRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_owner(
    State(use_case): State<RemoveOwnerUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<RemoveOwnerRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

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

// Category commands

pub async fn add_category(
    State(use_case): State<AddCategoryUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddCategoryRequest>,
) -> Result<(StatusCode, Json<AddCategoryResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_category(
    State(use_case): State<UpdateCategoryUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_category(
    State(use_case): State<DeleteCategoryUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
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

// Transaction commands

pub async fn add_transaction(
    State(use_case): State<AddTransactionUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddTransactionRequest>,
) -> Result<(StatusCode, Json<AddTransactionResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_transaction(
    State(use_case): State<UpdateTransactionUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_transaction(
    State(use_case): State<DeleteTransactionUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteTransactionRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Transaction queries

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

// Auth commands

pub async fn create_session_token(
    State(use_case): State<CreateSessionTokenUseCase>,
    Json(request): Json<CreateSessionTokenRequest>,
) -> Result<Json<CreateSessionTokenResponse>, ApiError> {
    let response = use_case.execute(request).await?;
    Ok(Json(response))
}
