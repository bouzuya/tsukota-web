use application::request::UpdateAccountRequest;
use application::use_case::UpdateAccountUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn update_account(
    State(use_case): State<UpdateAccountUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<UpdateAccountRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
