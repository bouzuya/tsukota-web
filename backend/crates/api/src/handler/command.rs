use application::request::AddCategoryRequest;
use application::request::AddOwnerRequest;
use application::request::AddTransactionRequest;
use application::request::CreateAccountRequest;
use application::request::CreateCustomTokenRequest;
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
use application::response::CreateCustomTokenResponse;
use application::use_case::AddCategoryUseCase;
use application::use_case::AddOwnerUseCase;
use application::use_case::AddTransactionUseCase;
use application::use_case::CreateAccountUseCase;
use application::use_case::CreateCustomTokenUseCase;
use application::use_case::DeleteAccountUseCase;
use application::use_case::DeleteCategoryUseCase;
use application::use_case::DeleteTransactionUseCase;
use application::use_case::RemoveOwnerUseCase;
use application::use_case::UpdateAccountUseCase;
use application::use_case::UpdateCategoryUseCase;
use application::use_case::UpdateTransactionUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

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

// Auth commands

pub async fn create_custom_token(
    State(use_case): State<CreateCustomTokenUseCase>,
    Json(request): Json<CreateCustomTokenRequest>,
) -> Result<Json<CreateCustomTokenResponse>, ApiError> {
    let response = use_case.execute(request).await?;
    Ok(Json(response))
}
