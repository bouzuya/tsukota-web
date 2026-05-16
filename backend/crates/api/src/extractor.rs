use application::UserId;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::cookie_jar::CookieJar;
use crate::error::AuthError;

/// 署名済み session Cookie から復元した認証済みユーザー
///
/// OIDC callback で発行された Cookie を `CookieJar` 経由で取り出し、
/// `UserId` にパースする。失敗時は 401 を返す。
pub(crate) struct CurrentUserId(pub(crate) UserId);

impl<S> FromRequestParts<S> for CurrentUserId
where
    S: Send + Sync,
    CookieJar: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError)?;
        let session = jar.get_session().ok_or(AuthError)?;
        let user_id = session.parse::<UserId>().map_err(|_| AuthError)?;
        Ok(CurrentUserId(user_id))
    }
}
