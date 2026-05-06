use async_trait::async_trait;

/// 認証フローの種別
///
/// callback ハンドラで signin / signup の判別に使う
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthFlow {
    SignIn,
    SignUp,
}

impl AuthFlow {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthFlow::SignIn => "signin",
            AuthFlow::SignUp => "signup",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "signin" => Some(AuthFlow::SignIn),
            "signup" => Some(AuthFlow::SignUp),
            _ => None,
        }
    }
}

/// 認可リクエストを開始するための情報
///
/// state / nonce / pkce_verifier はそれぞれ Cookie に保管して callback で照合する
#[derive(Clone, Debug)]
pub struct AuthorizationRequest {
    /// Google authorization endpoint への redirect 先 URL
    pub authorize_url: String,
    /// CSRF state
    pub state: String,
    /// OIDC nonce
    pub nonce: String,
    /// PKCE verifier (token 交換時に送出)
    pub pkce_verifier: String,
}

/// id_token から取り出すクレーム (現状 sub のみ使用)
#[derive(Clone, Debug)]
pub struct OidcClaims {
    /// Google アカウント識別子
    pub sub: String,
}

/// OIDC エラー型 (object-safe のため Box 化)
pub type OidcError = Box<dyn std::error::Error + Send + Sync>;

/// OIDC クライアント trait
///
/// authorize_url で認可 URL と state/nonce/pkce_verifier を発行し、
/// exchange_code で authorization code を id_token に交換しつつ署名/iss/aud/exp/nonce を検証する
#[async_trait]
pub trait OidcClient: Send + Sync {
    /// 認可 URL を生成する
    fn authorize_url(&self) -> Result<AuthorizationRequest, OidcError>;

    /// authorization code を id_token に交換し、検証済みクレームを返す
    ///
    /// # Arguments
    ///
    /// * `code` - Google から受け取った authorization code
    /// * `pkce_verifier` - signin/signup 時に発行した PKCE verifier
    /// * `nonce` - signin/signup 時に発行した nonce
    async fn exchange_code(
        &self,
        code: &str,
        pkce_verifier: &str,
        nonce: &str,
    ) -> Result<OidcClaims, OidcError>;
}
