use axum::Json;

use crate::error::ApiError;
use crate::extractor::AuthUser;

#[derive(serde::Serialize)]
pub struct GetMeResponse {
    user_id: String,
}

pub async fn get_me(AuthUser(user_id): AuthUser) -> Result<Json<GetMeResponse>, ApiError> {
    Ok(Json(GetMeResponse {
        user_id: user_id.to_string(),
    }))
}
