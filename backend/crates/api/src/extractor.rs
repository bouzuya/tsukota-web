use application::UserId;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::AuthError;

/// Authenticated user extracted from request
pub struct AuthUser(pub UserId);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // TODO: Implement proper JWT validation with Firebase Auth
            // For now, extract user_id from X-User-Id header (for development/testing)
            let user_id = parts
                .headers
                .get("X-User-Id")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<UserId>().ok())
                .ok_or(AuthError)?;

            Ok(AuthUser(user_id))
        }
    }
}
