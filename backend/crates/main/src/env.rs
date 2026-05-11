use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Env {
    /// ベースパス (デフォルト: "/lab/tsukota")
    pub(crate) base_path: String,
    /// 署名 Cookie の HMAC 鍵 (hex 文字列、≥64 byte 推奨)
    pub(crate) cookie_signing_secret: String,
    /// Firestore エミュレーターのホスト (例: "localhost:8080")
    pub(crate) firestore_emulator_host: Option<String>,
    /// Cloud Run では metadata server から取得するので None
    pub(crate) google_application_credentials: Option<String>,
    /// 本番環境かどうか (Cookie の Secure フラグ切替)
    pub(crate) is_prod: bool,
    /// Google OAuth クライアント ID
    pub(crate) oidc_client_id: String,
    /// Google OAuth クライアントシークレット
    pub(crate) oidc_client_secret: String,
    /// OIDC issuer の URL (デフォルト: "https://accounts.google.com")
    pub(crate) oidc_issuer_url: String,
    /// callback の絶対 URL (Google Console の Authorized redirect URIs に登録した値)
    pub(crate) oidc_redirect_uri: String,
    /// ポート番号 (デフォルト: 3000)
    pub(crate) port: u16,
    /// Firestore の接続先 プロジェクト ID
    pub(crate) project_id: String,
    /// 静的ファイルのディレクトリ
    pub(crate) public_dir: Option<PathBuf>,
    /// 署名に使用するサービスアカウントのメールアドレス
    pub(crate) service_account_email: String,
}

impl Env {
    pub(crate) fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = std::env::var("BASE_PATH").unwrap_or_else(|_| "/lab/tsukota".to_owned());
        let cookie_signing_secret = std::env::var("COOKIE_SIGNING_SECRET")
            .ok()
            .ok_or("COOKIE_SIGNING_SECRET not set")?;
        let firestore_emulator_host = std::env::var("FIRESTORE_EMULATOR_HOST").ok();
        let google_application_credentials = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
        let is_prod = std::env::var("IS_PROD")
            .ok()
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);
        let oidc_client_id = std::env::var("OIDC_CLIENT_ID")
            .ok()
            .ok_or("OIDC_CLIENT_ID not set")?;
        let oidc_client_secret = std::env::var("OIDC_CLIENT_SECRET")
            .ok()
            .ok_or("OIDC_CLIENT_SECRET not set")?;
        let oidc_issuer_url = std::env::var("OIDC_ISSUER_URL")
            .unwrap_or_else(|_| "https://accounts.google.com".to_owned());
        let oidc_redirect_uri = std::env::var("OIDC_REDIRECT_URI")
            .ok()
            .ok_or("OIDC_REDIRECT_URI not set")?;
        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000);
        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| std::env::var("PROJECT_ID"))
            .ok()
            .ok_or("GOOGLE_CLOUD_PROJECT nor PROJECT_ID not set")?;
        let public_dir = std::env::var("PUBLIC_DIR").ok().map(PathBuf::from);
        let service_account_email = std::env::var("SERVICE_ACCOUNT_EMAIL")
            .ok()
            .ok_or("SERVICE_ACCOUNT_EMAIL not set")?;
        Ok(Self {
            base_path,
            cookie_signing_secret,
            firestore_emulator_host,
            google_application_credentials,
            is_prod,
            oidc_client_id,
            oidc_client_secret,
            oidc_issuer_url,
            oidc_redirect_uri,
            port,
            project_id,
            public_dir,
            service_account_email,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Env;

    /// テスト用の COOKIE_SIGNING_SECRET (ダミーの hex 文字列)
    const TEST_COOKIE_SECRET: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    const TEST_OIDC_CLIENT_ID: &str = "client-id.apps.googleusercontent.com";
    const TEST_OIDC_CLIENT_SECRET: &str = "GOCSPX-test-secret";
    const TEST_OIDC_REDIRECT_URI: &str = "http://localhost:5173/lab/tsukota/auth/callback";

    /// 必須フィールドのみ設定したとき、省略可能フィールドはデフォルト値になる
    #[test]
    fn test_from_env_デフォルト値() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("BASE_PATH", None),
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("FIRESTORE_EMULATOR_HOST", None),
                ("GOOGLE_APPLICATION_CREDENTIALS", None),
                ("GOOGLE_CLOUD_PROJECT", None),
                ("IS_PROD", None),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_ISSUER_URL", None),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PORT", None),
                ("PROJECT_ID", Some("test-project")),
                ("PUBLIC_DIR", None),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                let env = Env::from_env().map_err(|e| anyhow::anyhow!("{e}"))?;

                assert_eq!(env.base_path, "/lab/tsukota");
                assert_eq!(env.cookie_signing_secret, TEST_COOKIE_SECRET);
                assert!(env.firestore_emulator_host.is_none());
                assert!(env.google_application_credentials.is_none());
                assert!(!env.is_prod);
                assert_eq!(env.oidc_client_id, TEST_OIDC_CLIENT_ID);
                assert_eq!(env.oidc_client_secret, TEST_OIDC_CLIENT_SECRET);
                assert_eq!(env.oidc_issuer_url, "https://accounts.google.com");
                assert_eq!(env.oidc_redirect_uri, TEST_OIDC_REDIRECT_URI);
                assert_eq!(env.port, 3000);
                assert_eq!(env.project_id, "test-project");
                assert!(env.public_dir.is_none());
                assert_eq!(env.service_account_email, "test@example.com");

                Ok(())
            },
        )
    }

    /// すべての環境変数を設定したとき、各フィールドに正しく反映される
    #[test]
    fn test_from_env_全フィールド設定() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("BASE_PATH", Some("/custom/path")),
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("FIRESTORE_EMULATOR_HOST", Some("localhost:8080")),
                (
                    "GOOGLE_APPLICATION_CREDENTIALS",
                    Some("/path/to/creds.json"),
                ),
                ("GOOGLE_CLOUD_PROJECT", None),
                ("IS_PROD", Some("true")),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_ISSUER_URL", Some("https://example.com")),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PORT", Some("8000")),
                ("PROJECT_ID", Some("my-project")),
                ("PUBLIC_DIR", Some("/var/www")),
                (
                    "SERVICE_ACCOUNT_EMAIL",
                    Some("sa@my-project.iam.gserviceaccount.com"),
                ),
            ],
            || {
                let env = Env::from_env().map_err(|e| anyhow::anyhow!("{e}"))?;

                assert_eq!(env.base_path, "/custom/path");
                assert!(env.is_prod);
                assert_eq!(env.oidc_issuer_url, "https://example.com");
                assert_eq!(env.port, 8000);
                assert_eq!(
                    env.google_application_credentials,
                    Some("/path/to/creds.json".to_owned())
                );
                assert_eq!(
                    env.firestore_emulator_host,
                    Some("localhost:8080".to_owned())
                );
                assert_eq!(env.project_id, "my-project");
                assert_eq!(env.public_dir, Some(PathBuf::from("/var/www")));
                assert_eq!(
                    env.service_account_email,
                    "sa@my-project.iam.gserviceaccount.com"
                );

                Ok(())
            },
        )
    }

    /// GOOGLE_CLOUD_PROJECT が設定されているとき PROJECT_ID より優先される
    #[test]
    fn test_from_env_google_cloud_project優先() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("GOOGLE_CLOUD_PROJECT", Some("gcp-project")),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PROJECT_ID", Some("other-project")),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                let env = Env::from_env().map_err(|e| anyhow::anyhow!("{e}"))?;
                assert_eq!(env.project_id, "gcp-project");
                Ok(())
            },
        )
    }

    /// GOOGLE_CLOUD_PROJECT も PROJECT_ID も未設定のときエラーになる
    #[test]
    fn test_from_env_project_id未設定() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("GOOGLE_CLOUD_PROJECT", None),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PROJECT_ID", None),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                assert!(Env::from_env().is_err());
                Ok(())
            },
        )
    }

    /// SERVICE_ACCOUNT_EMAIL が未設定のときエラーになる
    #[test]
    fn test_from_env_service_account_email未設定() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PROJECT_ID", Some("test-project")),
                ("SERVICE_ACCOUNT_EMAIL", None),
            ],
            || {
                assert!(Env::from_env().is_err());
                Ok(())
            },
        )
    }

    /// COOKIE_SIGNING_SECRET が未設定のときエラーになる
    #[test]
    fn test_from_env_cookie_signing_secret未設定() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("COOKIE_SIGNING_SECRET", None),
                ("OIDC_CLIENT_ID", Some(TEST_OIDC_CLIENT_ID)),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PROJECT_ID", Some("test-project")),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                assert!(Env::from_env().is_err());
                Ok(())
            },
        )
    }

    /// OIDC_CLIENT_ID が未設定のときエラーになる
    #[test]
    fn test_from_env_oidc_client_id未設定() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("COOKIE_SIGNING_SECRET", Some(TEST_COOKIE_SECRET)),
                ("OIDC_CLIENT_ID", None),
                ("OIDC_CLIENT_SECRET", Some(TEST_OIDC_CLIENT_SECRET)),
                ("OIDC_REDIRECT_URI", Some(TEST_OIDC_REDIRECT_URI)),
                ("PROJECT_ID", Some("test-project")),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                assert!(Env::from_env().is_err());
                Ok(())
            },
        )
    }
}
