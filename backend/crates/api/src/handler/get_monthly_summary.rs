use application::request::GetMonthlySummaryRequest;
use application::response::GetMonthlySummaryResponse;
use application::use_case::GetMonthlySummaryUseCase;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

use crate::error::ApiError;
use crate::extractor::AuthUser;

pub async fn get_monthly_summary(
    State(use_case): State<GetMonthlySummaryUseCase>,
    AuthUser(user_id): AuthUser,
    Path(account_id): Path<String>,
) -> Result<Json<GetMonthlySummaryResponse>, ApiError> {
    let request = GetMonthlySummaryRequest { account_id };
    let response = use_case.execute(&user_id, request).await?;
    Ok(Json(response))
}
