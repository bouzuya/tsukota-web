//! Cloud Run 環境向けの IAM signJwt API を使用したセッショントークン作成器
//!
//! Cloud Run では秘密鍵ファイルに直接アクセスできないため、
//! Google Cloud IAM の signJwt API を使用して JWT に署名する。

use application::SessionTokenCreator;
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
