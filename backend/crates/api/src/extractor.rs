use application::UserId;
use application::request::VerifySessionTokenRequest;
use application::use_case::VerifySessionTokenUseCase;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AuthError;

/// Authenticated user extracted from request
pub struct AuthUser(pub UserId);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    VerifySessionTokenUseCase: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Authorization ヘッダーから Bearer トークンを取得
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError)?;

        // "Bearer " プレフィックスを確認して取り除く
        let token = auth_header.strip_prefix("Bearer ").ok_or(AuthError)?;

        // トークンを検証して UID を取得
        let use_case = VerifySessionTokenUseCase::from_ref(state);
        let response = use_case
            .execute(VerifySessionTokenRequest {
                session_token: token.to_string(),
            })
            .map_err(|_| AuthError)?;

        // UID を UserId に変換
        let user_id = response.user_id.parse::<UserId>().map_err(|_| AuthError)?;

        Ok(AuthUser(user_id))
    }
}
