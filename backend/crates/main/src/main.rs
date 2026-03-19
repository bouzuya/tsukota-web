use std::path::PathBuf;
use std::sync::Arc;

use api::AppState;
use application::SessionTokenCreator;
use application::SessionTokenVerifier;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::DeviceRepository;
use application::repository::UserRepository;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use infra::FirestoreAccountRepository;
use infra::FirestoreClient;
use infra::FirestoreDeviceRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;
use infra::IamSessionTokenCreator;
use infra::IamSessionTokenVerifier;
use infra::PemSessionTokenCreator;
use infra::PemSessionTokenVerifier;
use infra::ServiceAccountCredentials;

#[derive(Debug)]
struct Env {
    /// ベースパス (デフォルト: "")
    base_path: String,
    /// Cloud Run では metadata server から取得するので None
    google_application_credentials: Option<String>,
    /// ポート番号 (デフォルト: 3000)
    port: u16,
    /// Firestore エミュレーターのホスト (例: "localhost:8080")
    firestore_emulator_host: Option<String>,
    /// Firestore の接続先 プロジェクト ID (None のときは Firebase Emulator)
    project_id: Option<String>,
    /// 静的ファイルのディレクトリ
    public_dir: PathBuf,
    /// 署名に使用するサービスアカウントのメールアドレス
    service_account_email: String,
}

impl Env {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let base_path = std::env::var("BASE_PATH").unwrap_or_else(|_| "/lab/tsukota".to_owned());
        let google_application_credentials = std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();
        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(3000);
        let firestore_emulator_host = std::env::var("FIRESTORE_EMULATOR_HOST").ok();
        let project_id = std::env::var("PROJECT_ID").ok();
        let public_dir = std::env::var("PUBLIC_DIR")
            .ok()
            .map(PathBuf::from)
            .ok_or("PUBLIC_DIR not set")?;
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

#[tokio::main]
async fn main() {
    let env = Env::from_env().expect("Failed to load environment variables");

    println!("{:?}", env);

    // Firestore uses FIRESTORE_EMULATOR_HOST if set
    let firestore = Firestore::new(FirestoreOptions {
        project_id: env.project_id.clone(),
    })
    .expect("Failed to initialize Firestore");

    let firestore_client = match (env.firestore_emulator_host, env.project_id) {
        (None, None) => {
            eprintln!("ERROR: Either FIRESTORE_EMULATOR_HOST or PROJECT_ID must be set");
            std::process::exit(1);
        }
        (None, Some(project_id)) => {
            FirestoreClient::connect(infra::DatabaseName::from_project_id(project_id).unwrap())
                .await
                .unwrap()
        }
        (Some(_emulator_host), None) => FirestoreClient::connect_with_emulator().await.unwrap(),
        (Some(_), Some(_)) => {
            eprintln!(
                "ERROR: Both FIRESTORE_EMULATOR_HOST and PROJECT_ID cannot be set at the same time"
            );
            std::process::exit(1);
        }
    };

    // Create repositories with Arc<dyn T>
    let account_repository: Arc<dyn AccountRepository> = Arc::new(FirestoreAccountRepository::new(
        firestore_client.clone(),
        firestore,
    ));
    let device_repository: Arc<dyn DeviceRepository> =
        Arc::new(FirestoreDeviceRepository::new(firestore_client.clone()));
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(FirestoreUserRepository::new(firestore_client.clone()));

    // Create projections with Arc<dyn T>
    let projection = FirestoreProjection::new(firestore_client);
    let account_projection: Arc<dyn AccountProjection> = Arc::new(projection.clone());
    let category_projection: Arc<dyn CategoryProjection> = Arc::new(projection.clone());
    let transaction_projection: Arc<dyn TransactionProjection> = Arc::new(projection);

    // Create session token creator and verifier
    let (creator, verifier): (Arc<dyn SessionTokenCreator>, Arc<dyn SessionTokenVerifier>) =
        match env.google_application_credentials {
            Some(google_application_credentials) => {
                match ServiceAccountCredentials::load(google_application_credentials) {
                    Ok(credentials) => {
                        let creator = PemSessionTokenCreator::new(&credentials.private_key)
                            .expect("Failed to create session token creator");
                        let verifier = PemSessionTokenVerifier::new(&credentials.private_key)
                            .expect("Failed to create session token verifier");
                        (Arc::new(creator), Arc::new(verifier))
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to load credentials: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                println!("GOOGLE_APPLICATION_CREDENTIALS not set, using IamSessionTokenCreator");
                let creator = IamSessionTokenCreator::new(env.service_account_email.clone());
                let verifier = IamSessionTokenVerifier::new(env.service_account_email.clone());
                (Arc::new(creator), Arc::new(verifier))
            }
        };

    // Create application state
    let state = AppState::new(
        account_repository,
        account_projection,
        category_projection,
        transaction_projection,
        device_repository,
        creator,
        verifier,
        user_repository,
    );

    // Run the server
    api::run(state, env.port, &env.public_dir, &env.base_path).await;
}
