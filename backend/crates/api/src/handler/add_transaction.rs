use application::request::AddTransactionRequest;
use application::response::AddTransactionResponse;
use application::use_case::AddTransactionUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn add_transaction(
    State(use_case): State<AddTransactionUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddTransactionRequest>,
) -> Result<(StatusCode, Json<AddTransactionResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
