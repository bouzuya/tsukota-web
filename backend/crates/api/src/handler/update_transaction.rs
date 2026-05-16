use application::request::UpdateTransactionRequest;
use application::use_case::UpdateTransactionUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn update_transaction(
    State(use_case): State<UpdateTransactionUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<UpdateTransactionRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
