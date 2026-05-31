//! OIDC 認証フローの auth router
//!
//! 4 ハンドラ (signin / signup / callback / signout) と
//! それらに必要な状態 `AuthState` を定義する。
//! Step 4c で main から `create_auth_router(...)` を呼んで main router に merge する。

#![allow(dead_code)]

use std::sync::Arc;

use application::AuthFlow;
use application::OidcClient;
use application::error::ApplicationError;
use application::request::SignInWithGoogleRequest;
use application::request::SignUpWithGoogleRequest;
use application::use_case::SignInWithGoogleUseCase;
use application::use_case::SignUpWithGoogleUseCase;
use axum::Json;
use axum::Router;
use axum::extract::FromRef;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use domain::GoogleUserId;
use serde::Deserialize;
use serde_json::json;

use crate::cookie_jar::BasePath;
use crate::cookie_jar::CookieJar;
use crate::cookie_jar::CookieKey;
use crate::cookie_jar::IsProd;
use crate::error::ApiError;

/// auth router 専用の state
///
/// `AppState` とは独立しており、main で別途構築して `create_auth_router` に渡す。
/// `CookieJar` extractor が要求する `CookieKey`/`BasePath`/`IsProd` も保持する。
#[derive(Clone)]
pub struct AuthState {
    oidc_client: Arc<dyn OidcClient>,
    sign_in_with_google: SignInWithGoogleUseCase,
    sign_up_with_google: SignUpWithGoogleUseCase,
    cookie_key: CookieKey,
    base_path: BasePath,
    is_prod: IsProd,
}

impl AuthState {
    pub fn new(
        oidc_client: Arc<dyn OidcClient>,
        sign_in_with_google: SignInWithGoogleUseCase,
        sign_up_with_google: SignUpWithGoogleUseCase,
        cookie_key: CookieKey,
        base_path: BasePath,
        is_prod: IsProd,
    ) -> Self {
        Self {
            oidc_client,
            sign_in_with_google,
            sign_up_with_google,
            cookie_key,
            base_path,
            is_prod,
        }
    }
}

impl FromRef<AuthState> for Arc<dyn OidcClient> {
    fn from_ref(state: &AuthState) -> Self {
        state.oidc_client.clone()
    }
}

impl FromRef<AuthState> for SignInWithGoogleUseCase {
    fn from_ref(state: &AuthState) -> Self {
        state.sign_in_with_google.clone()
    }
}

impl FromRef<AuthState> for SignUpWithGoogleUseCase {
    fn from_ref(state: &AuthState) -> Self {
        state.sign_up_with_google.clone()
    }
}

impl FromRef<AuthState> for CookieKey {
    fn from_ref(state: &AuthState) -> Self {
        state.cookie_key.clone()
    }
}

impl FromRef<AuthState> for BasePath {
    fn from_ref(state: &AuthState) -> Self {
        state.base_path.clone()
    }
}

impl FromRef<AuthState> for IsProd {
    fn from_ref(state: &AuthState) -> Self {
        state.is_prod
    }
}

/// `/auth/{signin,signup,callback,signout}` を扱う Router を構築する
///
/// 戻り値は `Router<()>` で main 側で他のルーターと `merge` または `nest` できる
pub fn create_auth_router(state: AuthState) -> Router<()> {
    Router::new()
        .route("/signin", get(signin))
        .route("/signup", get(signup))
        .route("/callback", get(callback))
        .route("/signout", post(signout))
        .with_state(state)
}

/// auth router 内で発生し得るエラー
enum AuthRouteError {
    /// Application 層のエラー (Forbidden / Unauthorized / Repository など)
    Application(ApplicationError),
    /// 必須クエリ/Cookie の欠損や state 不一致など
    BadRequest(String),
    /// OIDC クライアントの内部エラー
    Internal(String),
}

impl From<ApplicationError> for AuthRouteError {
    fn from(e: ApplicationError) -> Self {
        Self::Application(e)
    }
}

impl IntoResponse for AuthRouteError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "type": "auth_bad_request",
                    "title": msg,
                    "status": 400,
                })),
            )
                .into_response(),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "type": "auth_internal_error",
                    "title": msg,
                    "status": 500,
                })),
            )
                .into_response(),
            Self::Application(e) => ApiError(e).into_response(),
        }
    }
}

/// `/auth/signin` (GET): Google authorize URL に redirect
async fn signin(
    State(oidc_client): State<Arc<dyn OidcClient>>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), AuthRouteError> {
    let auth_request = oidc_client
        .authorize_url()
        .map_err(|e| AuthRouteError::Internal(e.to_string()))?;
    let url = auth_request.authorize_url.clone();
    let jar = jar.with_signin_cookies(&auth_request);
    Ok((jar, Redirect::temporary(&url)))
}

/// `/auth/signup` (GET): Google authorize URL に redirect (auth_flow=signup)
async fn signup(
    State(oidc_client): State<Arc<dyn OidcClient>>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), AuthRouteError> {
    let auth_request = oidc_client
        .authorize_url()
        .map_err(|e| AuthRouteError::Internal(e.to_string()))?;
    let url = auth_request.authorize_url.clone();
    let jar = jar.with_signup_cookies(&auth_request);
    Ok((jar, Redirect::temporary(&url)))
}

/// `/auth/callback` のクエリ引数
#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    error: Option<String>,
    state: Option<String>,
}

/// `/auth/callback` (GET): Google からの戻りで id_token 交換 → session 発行
async fn callback(
    State(oidc_client): State<Arc<dyn OidcClient>>,
    State(sign_in): State<SignInWithGoogleUseCase>,
    State(sign_up): State<SignUpWithGoogleUseCase>,
    State(base_path): State<BasePath>,
    jar: CookieJar,
    Query(query): Query<CallbackQuery>,
) -> Result<(CookieJar, Redirect), AuthRouteError> {
    if let Some(err) = query.error {
        return Err(AuthRouteError::BadRequest(format!("OIDC error: {err}")));
    }
    let code = query
        .code
        .ok_or_else(|| AuthRouteError::BadRequest("missing code".to_owned()))?;
    let state_param = query
        .state
        .ok_or_else(|| AuthRouteError::BadRequest("missing state".to_owned()))?;

    // Cookie 4 種が揃っているか確認
    let cookie_state = jar
        .get_state()
        .ok_or_else(|| AuthRouteError::BadRequest("missing oidc_state cookie".to_owned()))?;
    if cookie_state != state_param {
        return Err(AuthRouteError::BadRequest("state mismatch".to_owned()));
    }
    let cookie_nonce = jar
        .get_nonce()
        .ok_or_else(|| AuthRouteError::BadRequest("missing oidc_nonce cookie".to_owned()))?;
    let cookie_pkce = jar.get_pkce_verifier().ok_or_else(|| {
        AuthRouteError::BadRequest("missing oidc_pkce_verifier cookie".to_owned())
    })?;
    let cookie_flow = jar
        .get_auth_flow()
        .ok_or_else(|| AuthRouteError::BadRequest("missing auth_flow cookie".to_owned()))?;
    let auth_flow = AuthFlow::parse(&cookie_flow)
        .ok_or_else(|| AuthRouteError::BadRequest("invalid auth_flow".to_owned()))?;

    // id_token 交換 + 検証
    let claims = oidc_client
        .exchange_code(&code, &cookie_pkce, &cookie_nonce)
        .await
        .map_err(|e| AuthRouteError::BadRequest(format!("token exchange failed: {e}")))?;
    let google_user_id = claims
        .sub
        .parse::<GoogleUserId>()
        .map_err(|_| AuthRouteError::BadRequest("invalid sub claim".to_owned()))?;

    // signin / signup 分岐
    let user_id = match auth_flow {
        AuthFlow::SignIn => {
            sign_in
                .execute(SignInWithGoogleRequest { google_user_id })
                .await?
                .user_id
        }
        AuthFlow::SignUp => {
            sign_up
                .execute(SignUpWithGoogleRequest { google_user_id })
                .await?
                .user_id
        }
    };

    let jar = jar.with_session_cookies(user_id.to_string());
    Ok((jar, Redirect::temporary(&redirect_root(&base_path))))
}

/// `/auth/signout` (POST): session Cookie 削除 + login 画面へ redirect
async fn signout(State(base_path): State<BasePath>, jar: CookieJar) -> (CookieJar, Redirect) {
    let jar = jar.with_signout_cookies();
    (jar, Redirect::temporary(&redirect_login(&base_path)))
}

/// アプリ TOP への redirect 先 URL を組み立てる
fn redirect_root(base_path: &BasePath) -> String {
    if base_path.0.is_empty() {
        "/".to_owned()
    } else {
        format!("{}/", base_path.0)
    }
}

/// login 画面への redirect 先 URL を組み立てる
fn redirect_login(base_path: &BasePath) -> String {
    if base_path.0.is_empty() {
        "/login".to_owned()
    } else {
        format!("{}/login", base_path.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_root_with_base_path() {
        let bp = BasePath("/lab/tsukota".to_owned());
        assert_eq!(redirect_root(&bp), "/lab/tsukota/");
    }

    #[test]
    fn test_redirect_root_with_empty_base_path() {
        let bp = BasePath(String::new());
        assert_eq!(redirect_root(&bp), "/");
    }

    #[test]
    fn test_redirect_login_with_base_path() {
        let bp = BasePath("/lab/tsukota".to_owned());
        assert_eq!(redirect_login(&bp), "/lab/tsukota/login");
    }

    #[test]
    fn test_redirect_login_with_empty_base_path() {
        let bp = BasePath(String::new());
        assert_eq!(redirect_login(&bp), "/login");
    }
}
