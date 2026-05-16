use axum::Json;

use crate::error::ApiError;
use crate::extractor::CurrentUserId;

#[derive(serde::Serialize)]
pub struct GetMeResponse {
    user_id: String,
}

pub async fn get_me(
    CurrentUserId(user_id): CurrentUserId,
) -> Result<Json<GetMeResponse>, ApiError> {
    Ok(Json(GetMeResponse {
        user_id: user_id.to_string(),
    }))
}
