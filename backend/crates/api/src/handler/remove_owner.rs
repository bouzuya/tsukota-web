use application::request::RemoveOwnerRequest;
use application::use_case::RemoveOwnerUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn remove_owner(
    State(use_case): State<RemoveOwnerUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<RemoveOwnerRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
