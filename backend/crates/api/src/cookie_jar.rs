//! OIDC 認証フローで使う Cookie の名前・属性・ライフサイクルを `CookieJar` に閉じ込める
//!
//! 全 Cookie は `SignedCookieJar` (HMAC) で扱い、`HttpOnly` + `Secure (本番のみ)` +
//! `SameSite=Lax` + `Path=base_path` を明示的に付ける。
//!
//! 利用側 (`extractor.rs` や auth router ハンドラ) は `axum_extra` 直接ではなく
//! この `CookieJar` の `get_*` / `with_*` メソッドを経由する。Cookie 名はここに閉じる。

#![allow(dead_code)]

use std::convert::Infallible;

use application::AuthorizationRequest;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::IntoResponse;
use axum::response::IntoResponseParts;
use axum::response::Response;
use axum::response::ResponseParts;
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::Key;
use axum_extra::extract::cookie::SameSite;
use cookie::time::Duration;

/// 署名 Cookie の HMAC 鍵を保持する不透明な型
///
/// `axum_extra::extract::cookie::Key` をラップして `api` クレート外には
/// 実装型を漏らさない。crate 内では `pub(crate)` フィールド経由で内部の
/// `Key` を取り出して `SignedCookieJar` の extractor に渡す。
#[derive(Clone)]
pub struct CookieKey(pub(crate) Key);

impl CookieKey {
    /// 任意のバイト列から鍵を構築する
    ///
    /// バイト長は 64 byte 以上が必要 (axum-extra/cookie crate の制約)。
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Key::from(bytes))
    }

    /// 暗号学的乱数から鍵を生成する
    ///
    /// `Key::generate` に転送する。テストや開発環境で一時的な鍵が必要な
    /// ときに使う。
    pub fn generate() -> Self {
        Self(Key::generate())
    }

    /// マスター鍵の生バイト列を返す (64 byte)
    ///
    /// `Key::master` に転送する。`COOKIE_SIGNING_SECRET` 用に hex 化して
    /// 出力する CLI サブコマンドなどで利用する。
    pub fn master(&self) -> &[u8] {
        self.0.master()
    }
}

impl From<CookieKey> for Key {
    fn from(cookie_key: CookieKey) -> Self {
        cookie_key.0
    }
}

/// Cookie の `Path` 属性に設定するベースパス
///
/// 通常 `"/lab/tsukota"`。空文字なら Cookie の `Path` には `/` を設定する
#[derive(Clone, Debug)]
pub struct BasePath(pub String);

/// 本番環境かどうか。Cookie の `Secure` 属性切替に使う
#[derive(Clone, Copy, Debug)]
pub struct IsProd(pub bool);

/// 署名 Cookie を扱う axum の extractor
///
/// `axum_extra::extract::SignedCookieJar` を crate 内部に閉じ込める。
/// Cookie 名は impl 内 const としてここに集約し、外部には `get_session()` /
/// `with_signin_cookies(...)` のようなドメイン特化メソッドだけを公開する。
pub(crate) struct CookieJar {
    base_path: String,
    is_prod: bool,
    jar: SignedCookieJar<CookieKey>,
}

impl CookieJar {
    const COOKIE_AUTH_FLOW: &'static str = "auth_flow";
    const COOKIE_OIDC_NONCE: &'static str = "oidc_nonce";
    const COOKIE_OIDC_PKCE_VERIFIER: &'static str = "oidc_pkce_verifier";
    const COOKIE_OIDC_STATE: &'static str = "oidc_state";
    const COOKIE_SESSION: &'static str = "session";
    const SESSION_COOKIE_MAX_AGE_SECS: i64 = 60 * 60 * 24 * 30;

    fn cookie_path(&self) -> String {
        if self.base_path.is_empty() {
            "/".to_owned()
        } else {
            self.base_path.clone()
        }
    }

    /// 一時 Cookie (state / nonce / pkce_verifier / auth_flow) を生成する
    ///
    /// セッション cookie 相当 (Max-Age なし) で、ブラウザを閉じれば消える
    fn build_temp_cookie(&self, name: &'static str, value: String) -> Cookie<'static> {
        Cookie::build((name, value))
            .path(self.cookie_path())
            .http_only(true)
            .secure(self.is_prod)
            .same_site(SameSite::Lax)
            .build()
    }

    /// セッション Cookie を生成する。Max-Age は 30 日
    fn build_session_cookie(&self, value: String) -> Cookie<'static> {
        Cookie::build((Self::COOKIE_SESSION, value))
            .path(self.cookie_path())
            .http_only(true)
            .secure(self.is_prod)
            .same_site(SameSite::Lax)
            .max_age(Duration::seconds(Self::SESSION_COOKIE_MAX_AGE_SECS))
            .build()
    }

    /// Cookie 削除指示 Cookie (Max-Age=0 / 空値) を生成する
    fn build_removal_cookie(&self, name: &'static str) -> Cookie<'static> {
        Cookie::build((name, ""))
            .path(self.cookie_path())
            .http_only(true)
            .secure(self.is_prod)
            .same_site(SameSite::Lax)
            .max_age(Duration::ZERO)
            .build()
    }

    /// signin / signup フロー種別 ("signin" or "signup") を取得する
    pub(crate) fn get_auth_flow(&self) -> Option<String> {
        self.jar
            .get(Self::COOKIE_AUTH_FLOW)
            .map(|c| c.value().to_owned())
    }

    /// OIDC nonce を取得する
    pub(crate) fn get_nonce(&self) -> Option<String> {
        self.jar
            .get(Self::COOKIE_OIDC_NONCE)
            .map(|c| c.value().to_owned())
    }

    /// PKCE verifier を取得する
    pub(crate) fn get_pkce_verifier(&self) -> Option<String> {
        self.jar
            .get(Self::COOKIE_OIDC_PKCE_VERIFIER)
            .map(|c| c.value().to_owned())
    }

    /// CSRF state を取得する
    pub(crate) fn get_state(&self) -> Option<String> {
        self.jar
            .get(Self::COOKIE_OIDC_STATE)
            .map(|c| c.value().to_owned())
    }

    /// session Cookie の値 (アプリ内 UserId 文字列) を取得する
    pub(crate) fn get_session(&self) -> Option<String> {
        self.jar
            .get(Self::COOKIE_SESSION)
            .map(|c| c.value().to_owned())
    }

    /// signin フローの一時 4 種 Cookie を set した新しいジャーを返す
    pub(crate) fn with_signin_cookies(&self, auth_request: &AuthorizationRequest) -> Self {
        self.with_authorization_cookies("signin", auth_request)
    }

    /// signup フローの一時 4 種 Cookie を set した新しいジャーを返す
    pub(crate) fn with_signup_cookies(&self, auth_request: &AuthorizationRequest) -> Self {
        self.with_authorization_cookies("signup", auth_request)
    }

    fn with_authorization_cookies(
        &self,
        flow: &'static str,
        auth_request: &AuthorizationRequest,
    ) -> Self {
        let jar = self
            .jar
            .clone()
            .add(self.build_temp_cookie(Self::COOKIE_AUTH_FLOW, flow.to_owned()))
            .add(self.build_temp_cookie(Self::COOKIE_OIDC_STATE, auth_request.state.clone()))
            .add(self.build_temp_cookie(Self::COOKIE_OIDC_NONCE, auth_request.nonce.clone()))
            .add(self.build_temp_cookie(
                Self::COOKIE_OIDC_PKCE_VERIFIER,
                auth_request.pkce_verifier.clone(),
            ));
        Self {
            base_path: self.base_path.clone(),
            is_prod: self.is_prod,
            jar,
        }
    }

    /// 一時 4 種 Cookie を削除し、session Cookie を set した新しいジャーを返す
    pub(crate) fn with_session_cookies(&self, user_id: String) -> Self {
        let jar = self
            .jar
            .clone()
            .remove(self.build_removal_cookie(Self::COOKIE_OIDC_STATE))
            .remove(self.build_removal_cookie(Self::COOKIE_OIDC_NONCE))
            .remove(self.build_removal_cookie(Self::COOKIE_OIDC_PKCE_VERIFIER))
            .remove(self.build_removal_cookie(Self::COOKIE_AUTH_FLOW))
            .add(self.build_session_cookie(user_id));
        Self {
            base_path: self.base_path.clone(),
            is_prod: self.is_prod,
            jar,
        }
    }

    /// session Cookie を削除した新しいジャーを返す
    pub(crate) fn with_signout_cookies(&self) -> Self {
        let jar = self
            .jar
            .clone()
            .remove(self.build_removal_cookie(Self::COOKIE_SESSION));
        Self {
            base_path: self.base_path.clone(),
            is_prod: self.is_prod,
            jar,
        }
    }
}

impl<S> FromRequestParts<S> for CookieJar
where
    BasePath: FromRef<S>,
    CookieKey: FromRef<S>,
    IsProd: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::<CookieKey>::from_request_parts(parts, state).await?;
        let base_path = BasePath::from_ref(state);
        let is_prod = IsProd::from_ref(state);
        Ok(Self {
            base_path: base_path.0,
            is_prod: is_prod.0,
            jar,
        })
    }
}

impl IntoResponseParts for CookieJar {
    type Error = <SignedCookieJar<Key> as IntoResponseParts>::Error;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        self.jar.into_response_parts(res)
    }
}

impl IntoResponse for CookieJar {
    fn into_response(self) -> Response {
        self.jar.into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    struct AppStateForTest {
        cookie_key: CookieKey,
        base_path: BasePath,
        is_prod: IsProd,
    }

    impl FromRef<AppStateForTest> for BasePath {
        fn from_ref(state: &AppStateForTest) -> Self {
            state.base_path.clone()
        }
    }

    impl FromRef<AppStateForTest> for CookieKey {
        fn from_ref(state: &AppStateForTest) -> Self {
            state.cookie_key.clone()
        }
    }

    impl FromRef<AppStateForTest> for IsProd {
        fn from_ref(state: &AppStateForTest) -> Self {
            state.is_prod
        }
    }

    async fn make_empty_jar() -> anyhow::Result<CookieJar> {
        let request = Request::new(axum::body::Body::empty());
        let (mut parts, _) = request.into_parts();
        let state = AppStateForTest {
            cookie_key: CookieKey(Key::generate()),
            base_path: BasePath("/lab/tsukota".to_owned()),
            is_prod: IsProd(false),
        };
        Ok(CookieJar::from_request_parts(&mut parts, &state).await?)
    }

    fn make_auth_request() -> AuthorizationRequest {
        AuthorizationRequest {
            authorize_url: "https://accounts.google.com/o/oauth2/v2/auth?...".to_owned(),
            state: "test_state".to_owned(),
            nonce: "test_nonce".to_owned(),
            pkce_verifier: "test_pkce_verifier".to_owned(),
        }
    }

    #[test]
    fn test_generate_produces_different_keys() {
        let key1 = CookieKey::generate();
        let key2 = CookieKey::generate();
        // `Key` の `PartialEq` は constant-time 比較。乱数生成が動いていれば
        // 2 回の呼び出しで同じ鍵になる確率は無視できる
        assert_ne!(key1.0, key2.0);
    }

    #[tokio::test]
    async fn test_get_session_returns_none_when_empty() -> anyhow::Result<()> {
        let jar = make_empty_jar().await?;
        assert!(jar.get_session().is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_with_signin_cookies_sets_all_temp_cookies() -> anyhow::Result<()> {
        let jar = make_empty_jar()
            .await?
            .with_signin_cookies(&make_auth_request());
        assert_eq!(jar.get_auth_flow(), Some("signin".to_owned()));
        assert_eq!(jar.get_state(), Some("test_state".to_owned()));
        assert_eq!(jar.get_nonce(), Some("test_nonce".to_owned()));
        assert_eq!(
            jar.get_pkce_verifier(),
            Some("test_pkce_verifier".to_owned())
        );
        assert!(jar.get_session().is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_with_signup_cookies_sets_signup_flow() -> anyhow::Result<()> {
        let jar = make_empty_jar()
            .await?
            .with_signup_cookies(&make_auth_request());
        assert_eq!(jar.get_auth_flow(), Some("signup".to_owned()));
        Ok(())
    }

    #[tokio::test]
    async fn test_with_session_cookies_sets_session() -> anyhow::Result<()> {
        let jar = make_empty_jar()
            .await?
            .with_signin_cookies(&make_auth_request())
            .with_session_cookies("user-uuid-123".to_owned());
        assert_eq!(jar.get_session(), Some("user-uuid-123".to_owned()));
        Ok(())
    }

    #[tokio::test]
    async fn test_cookie_path_falls_back_to_root_when_base_path_empty() -> anyhow::Result<()> {
        let request = Request::new(axum::body::Body::empty());
        let (mut parts, _) = request.into_parts();
        let state = AppStateForTest {
            cookie_key: CookieKey(Key::generate()),
            base_path: BasePath("".to_owned()),
            is_prod: IsProd(false),
        };
        let jar = CookieJar::from_request_parts(&mut parts, &state).await?;
        assert_eq!(jar.cookie_path(), "/");
        Ok(())
    }
}
