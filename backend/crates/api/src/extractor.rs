use std::sync::Arc;

use application::token_signer::TokenVerifier;
use application::UserId;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AuthError;

/// Authenticated user extracted from request
pub struct AuthUser(pub UserId);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<dyn TokenVerifier>: FromRef<S>,
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
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError)?;

        // トークンを検証して UID を取得
        let verifier = Arc::<dyn TokenVerifier>::from_ref(state);
        let uid = verifier.verify(token).map_err(|_| AuthError)?;

        // UID を UserId に変換
        let user_id = uid.parse::<UserId>().map_err(|_| AuthError)?;

        Ok(AuthUser(user_id))
    }
}
