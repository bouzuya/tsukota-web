/// サービスアカウント認証情報の読み込みエラー
#[derive(Debug, thiserror::Error)]
pub enum ServiceAccountCredentialsError {
    #[error("GOOGLE_APPLICATION_CREDENTIALS 環境変数が設定されていません")]
    MissingCredentials,

    #[error("認証情報ファイルの読み込みに失敗しました: {0}")]
    ReadFile(#[from] std::io::Error),

    #[error("認証情報ファイルのパースに失敗しました: {0}")]
    ParseJson(#[from] serde_json::Error),

    #[error("認証情報に必要なフィールドが見つかりません: {0}")]
    MissingField(String),
}

/// サービスアカウント認証情報
///
/// Firebase カスタムトークンの署名に使用する
pub struct ServiceAccountCredentials {
    /// サービスアカウントのメールアドレス
    pub client_email: String,
    /// RSA 秘密鍵 (PEM 形式)
    pub private_key: String,
}

impl ServiceAccountCredentials {
    /// GOOGLE_APPLICATION_CREDENTIALS 環境変数からファイルパスを取得し、
    /// サービスアカウント認証情報を読み込む
    ///
    /// # Returns
    ///
    /// 読み込んだ認証情報
    ///
    /// # Note
    ///
    /// ローカル開発環境では GOOGLE_APPLICATION_CREDENTIALS が設定されている必要がある。
    /// 本番環境では別の方法（Secret Manager など）で認証情報を取得することを推奨。
    pub fn load(
        google_application_credentials: String,
    ) -> Result<Self, ServiceAccountCredentialsError> {
        let content = std::fs::read_to_string(&google_application_credentials)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let client_email = json["client_email"]
            .as_str()
            .ok_or_else(|| {
                ServiceAccountCredentialsError::MissingField("client_email".to_string())
            })?
            .to_string();

        let private_key = json["private_key"]
            .as_str()
            .ok_or_else(|| ServiceAccountCredentialsError::MissingField("private_key".to_string()))?
            .to_string();

        Ok(Self {
            client_email,
            private_key,
        })
    }
}
