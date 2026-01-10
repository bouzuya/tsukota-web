use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;
use application::request::AddCategoryRequest;
use application::request::AddOwnerRequest;
use application::request::AddTransactionRequest;
use application::request::CreateAccountRequest;
use application::request::DeleteAccountRequest;
use application::request::DeleteCategoryRequest;
use application::request::DeleteTransactionRequest;
use application::request::RemoveOwnerRequest;
use application::request::UpdateAccountRequest;
use application::request::UpdateCategoryRequest;
use application::request::UpdateTransactionRequest;
use application::response::AddCategoryResponse;
use application::response::AddTransactionResponse;
use application::response::CreateAccountResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::error::ApiError;
use crate::extractor::AuthUser;
use crate::state::AppState;

// Account commands

pub async fn create_account<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let response = state.create_account.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_account<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.update_account.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_account<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.delete_account.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_owner<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddOwnerRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.add_owner.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_owner<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<RemoveOwnerRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.remove_owner.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Category commands

pub async fn add_category<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddCategoryRequest>,
) -> Result<(StatusCode, Json<AddCategoryResponse>), ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let response = state.add_category.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_category<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.update_category.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_category<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteCategoryRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.delete_category.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Transaction commands

pub async fn add_transaction<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddTransactionRequest>,
) -> Result<(StatusCode, Json<AddTransactionResponse>), ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    let response = state.add_transaction.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn update_transaction<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.update_transaction.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_transaction<R, AP, CP, TP>(
    State(state): State<Arc<AppState<R, AP, CP, TP>>>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteTransactionRequest>,
) -> Result<StatusCode, ApiError>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    state.delete_transaction.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
