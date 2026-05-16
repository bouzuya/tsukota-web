use application::request::ListCategoriesRequest;
use application::response::ListCategoriesResponse;
use application::use_case::ListCategoriesUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn list_categories(
    State(use_case): State<ListCategoriesUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Path(account_id): Path<String>,
) -> Result<Json<ListCategoriesResponse>, ApiError> {
    let request = ListCategoriesRequest { account_id };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
