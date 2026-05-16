use application::request::DeleteTransactionRequest;
use application::use_case::DeleteTransactionUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn delete_transaction(
    State(use_case): State<DeleteTransactionUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<DeleteTransactionRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
