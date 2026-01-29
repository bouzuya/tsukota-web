use application::request::CreateSessionTokenRequest;
use application::response::CreateSessionTokenResponse;
use application::use_case::CreateSessionTokenUseCase;
use axum::Json;
use axum::extract::State;

use crate::error::ApiError;

pub async fn create_session_token(
    State(use_case): State<CreateSessionTokenUseCase>,
    Json(request): Json<CreateSessionTokenRequest>,
) -> Result<Json<CreateSessionTokenResponse>, ApiError> {
    let response = use_case.execute(request).await?;
    Ok(Json(response))
}
