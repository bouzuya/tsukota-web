use application::request::CreateAccountRequest;
use application::response::CreateAccountResponse;
use application::use_case::CreateAccountUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn create_account(
    State(use_case): State<CreateAccountUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
