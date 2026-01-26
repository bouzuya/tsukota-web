//! Cloud Run 環境向けの IAM signJwt API を使用したセッショントークン作成器・検証器
//!
//! Cloud Run では秘密鍵ファイルに直接アクセスできないため、
//! Google Cloud IAM の signJwt API を使用して JWT に署名する。
//! 検証時は Google の公開エンドポイントから公開鍵を取得する。

use std::collections::BTreeMap;

use application::SessionTokenCreator;
use application::SessionTokenVerifier;
use google_cloud_iam_credentials_v1::client::IAMCredentials;

use crate::signer::SessionTokenClaims;

/// IAM signJwt API を使用したトークン作成エラー
#[derive(Debug)]
pub enum IamSessionTokenCreateError {
    /// UID が空または 128 文字を超えている
    InvalidUid,
    /// システム時刻エラー
    SystemTimeError,
    /// IAM クライアントの作成に失敗
    ClientBuildError(String),
    /// JWT の署名に失敗
    SignJwtError(String),
    /// JSON シリアライズエラー
    JsonError(String),
}

impl std::fmt::Display for IamSessionTokenCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IamSessionTokenCreateError::InvalidUid => {
                write!(f, "UID must be between 1 and 128 characters")
            }
            IamSessionTokenCreateError::SystemTimeError => {
                write!(f, "Failed to get system time")
            }
            IamSessionTokenCreateError::ClientBuildError(e) => {
                write!(f, "Failed to build IAM client: {}", e)
            }
            IamSessionTokenCreateError::SignJwtError(e) => {
                write!(f, "Failed to sign JWT: {}", e)
            }
            IamSessionTokenCreateError::JsonError(e) => {
                write!(f, "JSON serialization error: {}", e)
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
    /// * `uid` - ユーザー識別子 (1-128 文字)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    ///
    /// # Returns
    ///
    /// JWT 形式のセッショントークン
    async fn create_impl(&self, uid: &str, now: u64) -> Result<String, IamSessionTokenCreateError> {
        // UID の長さを検証
        if !(1..=128).contains(&uid.len()) {
            return Err(IamSessionTokenCreateError::InvalidUid);
        }

        // クレームを作成
        let claims = SessionTokenClaims {
            iss: self.service_account_email.clone(),
            sub: self.service_account_email.clone(),
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_owned(),
            iat: now,
            exp: now + 3600,
            uid: uid.to_string(),
        };

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
        uid: &str,
        now: u64,
    ) -> Result<String, application::session_token::CreatorError> {
        self.create_impl(uid, now)
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
    /// キー ID が見つからない
    KeyIdNotFound(String),
    /// X.509 証明書のパースに失敗
    CertificateParseError(String),
    /// JWT デコードに失敗
    JwtDecodeError(String),
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
            IamSessionTokenVerifyError::KeyIdNotFound(kid) => {
                write!(f, "Key ID not found: {}", kid)
            }
            IamSessionTokenVerifyError::CertificateParseError(e) => {
                write!(f, "Failed to parse X.509 certificate: {}", e)
            }
            IamSessionTokenVerifyError::JwtDecodeError(e) => {
                write!(f, "JWT decode error: {}", e)
            }
        }
    }
}

impl std::error::Error for IamSessionTokenVerifyError {}

/// Cloud Run 向け IAM ベースのセッショントークン検証器
///
/// Google の公開エンドポイントから公開鍵を取得して JWT を検証する。
/// 公開鍵は `https://www.googleapis.com/robot/v1/metadata/x509/{SERVICE_ACCOUNT_EMAIL}` から取得する。
#[derive(Clone)]
pub struct IamSessionTokenVerifier {
    service_account_email: String,
}

impl IamSessionTokenVerifier {
    /// 新しい IamSessionTokenVerifier インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `service_account_email` - サービスアカウントのメールアドレス
    pub fn new(service_account_email: String) -> Self {
        Self {
            service_account_email,
        }
    }

    /// Google の公開エンドポイントから公開鍵証明書を取得する
    async fn fetch_public_keys(
        &self,
    ) -> Result<BTreeMap<String, String>, IamSessionTokenVerifyError> {
        let url = format!(
            "https://www.googleapis.com/robot/v1/metadata/x509/{}",
            urlencoding::encode(&self.service_account_email)
        );

        let client = reqwest::Client::new();
        let response = client
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

        let keys: BTreeMap<String, String> = response
            .json()
            .await
            .map_err(|e| IamSessionTokenVerifyError::FetchPublicKeyError(e.to_string()))?;

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

    /// X.509 証明書から公開鍵を抽出する
    fn extract_public_key_from_cert(
        cert_pem: &str,
    ) -> Result<jsonwebtoken::DecodingKey, IamSessionTokenVerifyError> {
        // X.509 証明書から公開鍵を抽出
        use x509_cert::der::Decode;

        // PEM からDER に変換
        let pem = pem::parse(cert_pem)
            .map_err(|e| IamSessionTokenVerifyError::CertificateParseError(e.to_string()))?;

        // X.509 証明書をパース
        let cert = x509_cert::Certificate::from_der(pem.contents())
            .map_err(|e| IamSessionTokenVerifyError::CertificateParseError(e.to_string()))?;

        // 公開鍵情報を取得
        let spki = cert.tbs_certificate.subject_public_key_info;

        // SPKI を DER 形式でエンコード
        use x509_cert::der::Encode;
        let spki_der = spki
            .to_der()
            .map_err(|e| IamSessionTokenVerifyError::CertificateParseError(e.to_string()))?;

        // jsonwebtoken の DecodingKey を作成
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_der(&spki_der);

        Ok(decoding_key)
    }

    /// トークンを検証して UID を取得する（非同期版）
    ///
    /// # Arguments
    ///
    /// * `token` - 検証する JWT トークン
    ///
    /// # Returns
    ///
    /// トークンに含まれる UID
    pub async fn verify_async(&self, token: &str) -> Result<String, IamSessionTokenVerifyError> {
        // JWT ヘッダーからキー ID を取得
        let kid = Self::get_key_id_from_token(token)?;

        // 公開鍵証明書を取得
        let public_keys = self.fetch_public_keys().await?;

        // キー ID に対応する証明書を取得
        let cert_pem = public_keys.get(&kid).ok_or_else(|| {
            IamSessionTokenVerifyError::KeyIdNotFound(format!("Key ID '{}' not found", kid))
        })?;

        // 証明書から公開鍵を抽出
        let decoding_key = Self::extract_public_key_from_cert(cert_pem)?;

        // JWT を検証
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&["https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit"]);
        validation.set_issuer(&[&self.service_account_email]);
        validation.set_required_spec_claims::<&str>(&[]);

        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(token, &decoding_key, &validation)
                .map_err(|e| IamSessionTokenVerifyError::JwtDecodeError(e.to_string()))?;

        Ok(token_data.claims.uid)
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
