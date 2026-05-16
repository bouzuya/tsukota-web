use application::request::GetAccountRequest;
use application::response::GetAccountResponse;
use application::use_case::GetAccountUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

pub async fn get_account(
    State(use_case): State<GetAccountUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Path(account_id): Path<String>,
) -> Result<Json<GetAccountResponse>, ApiError> {
    let request = GetAccountRequest { account_id };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
