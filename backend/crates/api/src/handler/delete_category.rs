use application::request::DeleteCategoryRequest;
use application::use_case::DeleteCategoryUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn delete_category(
    State(use_case): State<DeleteCategoryUseCase>,
    AuthUser(user_id): AuthUser,
    Json(request): Json<DeleteCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
