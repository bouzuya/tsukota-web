use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Env {
    /// ベースパス (デフォルト: "/lab/tsukota")
    pub(crate) base_path: String,
    /// Firestore エミュレーターのホスト (例: "localhost:8080")
    pub(crate) firestore_emulator_host: Option<String>,
    /// Cloud Run では metadata server から取得するので None
    pub(crate) google_application_credentials: Option<String>,
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
        let firestore_emulator_host = std::env::var("FIRESTORE_EMULATOR_HOST").ok();
        let google_application_credentials = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
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
            firestore_emulator_host,
            google_application_credentials,
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

    /// 必須フィールドのみ設定したとき、省略可能フィールドはデフォルト値になる
    #[test]
    fn test_from_env_デフォルト値() -> anyhow::Result<()> {
        temp_env::with_vars(
            [
                ("BASE_PATH", None),
                ("FIRESTORE_EMULATOR_HOST", None),
                ("GOOGLE_APPLICATION_CREDENTIALS", None),
                ("GOOGLE_CLOUD_PROJECT", None),
                ("PORT", None),
                ("PROJECT_ID", Some("test-project")),
                ("PUBLIC_DIR", None),
                ("SERVICE_ACCOUNT_EMAIL", Some("test@example.com")),
            ],
            || {
                let env = Env::from_env().map_err(|e| anyhow::anyhow!("{e}"))?;

                assert_eq!(env.base_path, "/lab/tsukota");
                assert!(env.firestore_emulator_host.is_none());
                assert!(env.google_application_credentials.is_none());
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
                ("FIRESTORE_EMULATOR_HOST", Some("localhost:8080")),
                (
                    "GOOGLE_APPLICATION_CREDENTIALS",
                    Some("/path/to/creds.json"),
                ),
                ("GOOGLE_CLOUD_PROJECT", None),
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
                ("GOOGLE_CLOUD_PROJECT", Some("gcp-project")),
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
                ("GOOGLE_CLOUD_PROJECT", None),
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
                ("PROJECT_ID", Some("test-project")),
                ("SERVICE_ACCOUNT_EMAIL", None),
            ],
            || {
                assert!(Env::from_env().is_err());
                Ok(())
            },
        )
    }
}
