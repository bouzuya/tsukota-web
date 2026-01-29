use application::request::CreateAccountRequest;
use application::response::CreateAccountResponse;
use application::use_case::CreateAccountUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn create_account(
    State(use_case): State<CreateAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<CreateAccountResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
