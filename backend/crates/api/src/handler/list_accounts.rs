use application::request::ListAccountsRequest;
use application::response::ListAccountsResponse;
use application::use_case::ListAccountsUseCase;
use axum::Json;
use axum::extract::State;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn list_accounts(
    State(use_case): State<ListAccountsUseCase>,
    CurrentUserId(user_id): CurrentUserId,
) -> Result<Json<ListAccountsResponse>, ApiError> {
    let request = ListAccountsRequest {};
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
