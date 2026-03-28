use std::sync::Arc;

mod env;

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
use env::Env;
use infra::FirestoreAccountRepository;
use infra::FirestoreDeviceRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;
use infra::IamSessionTokenCreator;
use infra::IamSessionTokenVerifier;
use infra::PemSessionTokenCreator;
use infra::PemSessionTokenVerifier;
use infra::ServiceAccountCredentials;
use tracing::info;

#[tokio::main]
async fn main() {
    // tracing の初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let env = Env::from_env().expect("Failed to load environment variables");

    tracing::info!(?env, "環境変数を読み込みました");

    // Firestore uses FIRESTORE_EMULATOR_HOST if set
    let firestore = Firestore::new(FirestoreOptions {
        project_id: Some(env.project_id.clone()),
    })
    .expect("Failed to initialize Firestore");

    // Create repositories with Arc<dyn T>
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(firestore.clone()));
    let device_repository: Arc<dyn DeviceRepository> =
        Arc::new(FirestoreDeviceRepository::new(firestore.clone()));
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(FirestoreUserRepository::new(firestore.clone()));

    // Create projections with Arc<dyn T>
    let projection = FirestoreProjection::new(firestore.clone());
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
                        tracing::error!("認証情報の読み込みに失敗しました: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                tracing::info!(
                    "GOOGLE_APPLICATION_CREDENTIALS が未設定のため IamSessionTokenCreator を使用します"
                );
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

    tracing::info!("アプリケーションの初期化が完了しました");

    // Run the server
    api::run(state, env.port, env.public_dir.as_deref(), &env.base_path).await;
}
