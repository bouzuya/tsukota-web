use application::request::AddOwnerRequest;
use application::use_case::AddOwnerUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn add_owner(
    State(use_case): State<AddOwnerUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<AddOwnerRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
