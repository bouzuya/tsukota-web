use application::request::ListTransactionsRequest;
use application::response::ListTransactionsResponse;
use application::use_case::ListTransactionsUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

const DEFAULT_PAGE_SIZE: usize = 20;

#[derive(Debug, Deserialize)]
pub struct ListTransactionsParams {
    pub after: Option<String>,
}

pub async fn list_transactions(
    State(use_case): State<ListTransactionsUseCase>,
    CurrentUserId(user_id): CurrentUserId,
    Path(account_id): Path<String>,
    Query(params): Query<ListTransactionsParams>,
) -> Result<Json<ListTransactionsResponse>, ApiError> {
    let request = ListTransactionsRequest {
        account_id,
        cursor: params.after,
        limit: DEFAULT_PAGE_SIZE,
    };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
