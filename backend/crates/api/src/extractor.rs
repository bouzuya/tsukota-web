use application::UserId;
use application::request::VerifySessionTokenRequest;
use application::use_case::VerifySessionTokenUseCase;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::cookie_jar::CookieJar;
use crate::error::AuthError;

/// Authenticated user extracted from request
///
/// Step B で削除予定。それまでの暫定で dead_code 警告を抑制している
#[allow(dead_code)]
pub(crate) struct AuthUser(pub(crate) UserId);

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
            .await
            .map_err(|_| AuthError)?;

        // UID を UserId に変換
        let user_id = response.user_id.parse::<UserId>().map_err(|_| AuthError)?;

        Ok(AuthUser(user_id))
    }
}

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
