use application::request::ExportTransactionsRequest;
use application::response::ExportTransactionsResponse;
use application::use_case::ExportTransactionsUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use serde::Deserialize;

use crate::error::ApiError;
use crate::extractor::AuthUser;

#[derive(Debug, Deserialize)]
pub struct ExportTransactionsParams {
    pub year: i32,
    pub month: u32,
}

pub async fn export_transactions(
    State(use_case): State<ExportTransactionsUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
    Query(params): Query<ExportTransactionsParams>,
) -> Result<Json<ExportTransactionsResponse>, ApiError> {
    let request = ExportTransactionsRequest {
        account_id,
        year: params.year,
        month: params.month,
    };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
