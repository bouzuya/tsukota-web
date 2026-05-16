use application::request::AddCategoryRequest;
use application::response::AddCategoryResponse;
use application::use_case::AddCategoryUseCase;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn add_category(
    State(use_case): State<AddCategoryUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Json(request): Json<AddCategoryRequest>,
) -> Result<(StatusCode, Json<AddCategoryResponse>), ApiError> {
    let response = use_case.execute(&user_id, request).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
