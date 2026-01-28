//! Cloud Run 環境向けの IAM signJwt API を使用したセッショントークン作成器・検証器
//!
//! Cloud Run では秘密鍵ファイルに直接アクセスできないため、
//! Google Cloud IAM の signJwt API を使用して JWT に署名する。
//! 検証時は Google の JWK エンドポイントから公開鍵を取得する。

use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;

use application::SessionTokenCreator;
use application::SessionTokenVerifier;
use google_cloud_iam_credentials_v1::client::IAMCredentials;

use super::claims::SessionTokenClaims;

/// IAM signJwt API を使用したトークン作成エラー
#[derive(Debug)]
pub enum IamSessionTokenCreateError {
    /// IAM クライアントの作成に失敗
    ClientBuildError(String),
    /// JSON シリアライズエラー
    JsonError(String),
    /// JWT の署名に失敗
    SignJwtError(String),
}

impl std::fmt::Display for IamSessionTokenCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IamSessionTokenCreateError::ClientBuildError(e) => {
                write!(f, "Failed to build IAM client: {}", e)
            }
            IamSessionTokenCreateError::JsonError(e) => {
                write!(f, "JSON serialization error: {}", e)
            }
            IamSessionTokenCreateError::SignJwtError(e) => {
                write!(f, "Failed to sign JWT: {}", e)
            }
        }
    }
}

impl std::error::Error for IamSessionTokenCreateError {}

/// Cloud Run 向け IAM ベースのセッショントークン作成器
///
/// Google Cloud IAM の signJwt API を使用して JWT に署名する。
/// Cloud Run のサービスアカウントが `iam.serviceAccounts.signJwt` 権限を
/// 持っている必要がある。
///
/// <https://docs.cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/signJwt>
#[derive(Clone)]
pub struct IamSessionTokenCreator {
    service_account_email: String,
}

impl IamSessionTokenCreator {
    /// 新しい IamSessionTokenCreator インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `service_account_email` - サービスアカウントのメールアドレス
    pub fn new(service_account_email: String) -> Self {
        Self {
            service_account_email,
        }
    }

    /// IAM signJwt API を使用して JWT に署名する
    async fn sign_jwt(&self, payload: &str) -> Result<String, IamSessionTokenCreateError> {
        let client = IAMCredentials::builder()
            .build()
            .await
            .map_err(|e| IamSessionTokenCreateError::ClientBuildError(e.to_string()))?;

        let name = format!("projects/-/serviceAccounts/{}", self.service_account_email);

        let response = client
            .sign_jwt()
            .set_name(&name)
            .set_payload(payload)
            .send()
            .await
            .map_err(|e| IamSessionTokenCreateError::SignJwtError(e.to_string()))?;

        Ok(response.signed_jwt)
    }

    /// 指定されたユーザーのセッショントークンを作成する（非同期版）
    ///
    /// # Arguments
    ///
    /// * `user_id` - ユーザー識別子
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    ///
    /// # Returns
    ///
    /// JWT 形式のセッショントークン
    async fn create_impl(
        &self,
        user_id: &str,
        now: u64,
    ) -> Result<String, IamSessionTokenCreateError> {
        // クレームを作成
        let claims = SessionTokenClaims::new(user_id.to_owned(), now);

        // JWT ペイロードを JSON 文字列に変換
        let payload = serde_json::to_string(&claims)
            .map_err(|e| IamSessionTokenCreateError::JsonError(e.to_string()))?;

        // IAM API で署名
        self.sign_jwt(&payload).await
    }
}

#[async_trait::async_trait]
impl SessionTokenCreator for IamSessionTokenCreator {
    async fn create(
        &self,
        user_id: &str,
        now: u64,
    ) -> Result<String, application::session_token::CreatorError> {
        self.create_impl(user_id, now)
            .await
            .map_err(|e| Box::new(e) as application::session_token::CreatorError)
    }
}

/// IAM 公開鍵を使用したトークン検証エラー
#[derive(Debug)]
pub enum IamSessionTokenVerifyError {
    /// HTTP リクエストエラー
    HttpError(String),
    /// 公開鍵の取得に失敗
    FetchPublicKeyError(String),
    /// JWK のパースに失敗
    JwkParseError(String),
    /// JWT デコードに失敗
    JwtDecodeError(String),
    /// キー ID が見つからない
    KeyIdNotFound(String),
}

impl std::fmt::Display for IamSessionTokenVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IamSessionTokenVerifyError::HttpError(e) => {
                write!(f, "HTTP request error: {}", e)
            }
            IamSessionTokenVerifyError::FetchPublicKeyError(e) => {
                write!(f, "Failed to fetch public key: {}", e)
            }
            IamSessionTokenVerifyError::JwkParseError(e) => {
                write!(f, "Failed to parse JWK: {}", e)
            }
            IamSessionTokenVerifyError::JwtDecodeError(e) => {
                write!(f, "JWT decode error: {}", e)
            }
            IamSessionTokenVerifyError::KeyIdNotFound(kid) => {
                write!(f, "Key ID not found: {}", kid)
            }
        }
    }
}

impl std::error::Error for IamSessionTokenVerifyError {}

/// JWK レスポンスの構造体
#[derive(Debug, serde::Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

/// 個別の JWK
#[derive(Clone, Debug, serde::Deserialize)]
struct Jwk {
    /// キー ID
    kid: String,
    /// RSA modulus (Base64URL エンコード)
    n: String,
    /// RSA exponent (Base64URL エンコード)
    e: String,
}

/// キャッシュされた JWK セット
struct CachedJwkSet {
    /// 取得した JWK リスト
    keys: Vec<Jwk>,
    /// 取得時刻
    fetched_at: Instant,
}

/// Cloud Run 向け IAM ベースのセッショントークン検証器
///
/// Google の JWK エンドポイントから公開鍵を取得して JWT を検証する。
/// 公開鍵は `https://www.googleapis.com/service_accounts/v1/metadata/jwk/{SERVICE_ACCOUNT_EMAIL}` から取得する。
/// 取得した鍵は 1 時間キャッシュされる。
#[derive(Clone)]
pub struct IamSessionTokenVerifier {
    /// HTTP クライアント
    client: reqwest::Client,
    /// JWK キャッシュ
    jwk_cache: Arc<RwLock<Option<CachedJwkSet>>>,
    /// サービスアカウントのメールアドレス
    service_account_email: String,
}

impl IamSessionTokenVerifier {
    /// キャッシュの有効期間（1 時間）
    const CACHE_DURATION: Duration = Duration::from_secs(60 * 60);

    /// 新しい IamSessionTokenVerifier インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `service_account_email` - サービスアカウントのメールアドレス
    pub fn new(service_account_email: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            jwk_cache: Arc::new(RwLock::new(None)),
            service_account_email,
        }
    }

    /// Google の JWK エンドポイントから公開鍵を取得する
    async fn fetch_jwk_set(&self) -> Result<Vec<Jwk>, IamSessionTokenVerifyError> {
        let url = format!(
            "https://www.googleapis.com/service_accounts/v1/metadata/jwk/{}",
            urlencoding::encode(&self.service_account_email)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| IamSessionTokenVerifyError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IamSessionTokenVerifyError::FetchPublicKeyError(format!(
                "HTTP status: {}",
                response.status()
            )));
        }

        let jwk_set: JwkSet = response
            .json()
            .await
            .map_err(|e| IamSessionTokenVerifyError::FetchPublicKeyError(e.to_string()))?;

        Ok(jwk_set.keys)
    }

    /// キャッシュから JWK を取得するか、新しく取得してキャッシュする
    async fn get_jwk_set(&self) -> Result<Vec<Jwk>, IamSessionTokenVerifyError> {
        // キャッシュをチェック
        {
            let cache = self.jwk_cache.read().unwrap();
            if let Some(cached) = cache.as_ref()
                && cached.fetched_at.elapsed() < Self::CACHE_DURATION
            {
                return Ok(cached.keys.clone());
            }
        }

        // キャッシュが無効または存在しない場合は新しく取得
        let keys = self.fetch_jwk_set().await?;

        // キャッシュを更新
        {
            let mut cache = self.jwk_cache.write().unwrap();
            *cache = Some(CachedJwkSet {
                keys: keys.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(keys)
    }

    /// JWT ヘッダーからキー ID を取得する
    fn get_key_id_from_token(token: &str) -> Result<String, IamSessionTokenVerifyError> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|e| IamSessionTokenVerifyError::JwtDecodeError(e.to_string()))?;

        header
            .kid
            .ok_or_else(|| IamSessionTokenVerifyError::KeyIdNotFound("No kid in header".to_owned()))
    }

    /// JWK から DecodingKey を作成する
    fn create_decoding_key(
        jwk: &Jwk,
    ) -> Result<jsonwebtoken::DecodingKey, IamSessionTokenVerifyError> {
        jsonwebtoken::DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|e| IamSessionTokenVerifyError::JwkParseError(e.to_string()))
    }

    /// トークンを検証してユーザー ID を取得する（非同期版）
    ///
    /// exp, aud, iss を検証する
    ///
    /// # Arguments
    ///
    /// * `token` - 検証する JWT トークン
    ///
    /// # Returns
    ///
    /// トークンに含まれるユーザー ID
    pub async fn verify_async(&self, token: &str) -> Result<String, IamSessionTokenVerifyError> {
        // JWT ヘッダーからキー ID を取得
        let kid = Self::get_key_id_from_token(token)?;

        // JWK セットを取得
        let jwk_set = self.get_jwk_set().await?;

        // キー ID に対応する JWK を取得
        let jwk = jwk_set.iter().find(|k| k.kid == kid).ok_or_else(|| {
            IamSessionTokenVerifyError::KeyIdNotFound(format!("Key ID '{}' not found", kid))
        })?;

        // JWK から DecodingKey を作成
        let decoding_key = Self::create_decoding_key(jwk)?;

        // JWT を検証（exp, aud, iss を検証）
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[SessionTokenClaims::AUDIENCE]);
        validation.set_issuer(&[SessionTokenClaims::ISSUER]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "sub"]);
        validation.validate_exp = true;

        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(token, &decoding_key, &validation)
                .map_err(|e| IamSessionTokenVerifyError::JwtDecodeError(e.to_string()))?;

        Ok(token_data.claims.user_id().to_owned())
    }
}

#[async_trait::async_trait]
impl SessionTokenVerifier for IamSessionTokenVerifier {
    async fn verify(
        &self,
        token: &str,
    ) -> Result<String, application::session_token::VerifierError> {
        self.verify_async(token)
            .await
            .map_err(|e| Box::new(e) as application::session_token::VerifierError)
    }
}
