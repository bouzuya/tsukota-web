use application::request::DeleteAccountRequest;
use application::use_case::DeleteAccountUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn delete_account(
    State(use_case): State<DeleteAccountUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
