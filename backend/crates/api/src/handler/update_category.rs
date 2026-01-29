use application::request::UpdateCategoryRequest;
use application::use_case::UpdateCategoryUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn update_category(
    State(use_case): State<UpdateCategoryUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
