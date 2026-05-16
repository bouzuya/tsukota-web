use application::request::DeleteCategoryRequest;
use application::use_case::DeleteCategoryUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn delete_category(
    State(use_case): State<DeleteCategoryUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<DeleteCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    use_case.execute(&user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}
