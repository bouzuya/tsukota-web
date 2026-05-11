use std::sync::Arc;

mod env;

use api::AppState;
use api::AuthState;
use api::BasePath;
use api::CookieKey;
use api::IsProd;
use application::SessionTokenCreator;
use application::SessionTokenVerifier;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::MonthlySummaryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::DeviceRepository;
use application::repository::GoogleUserMapRepository;
use application::repository::UserRepository;
use application::use_case::SignInWithGoogleUseCase;
use application::use_case::SignUpWithGoogleUseCase;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use env::Env;
use infra::FirestoreAccountRepository;
use infra::FirestoreDeviceRepository;
use infra::FirestoreGoogleUserMapRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;
use infra::GoogleOidcClient;
use infra::IamSessionTokenCreator;
use infra::IamSessionTokenVerifier;
use infra::PemSessionTokenCreator;
use infra::PemSessionTokenVerifier;
use infra::ServiceAccountCredentials;

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
    match env.firestore_emulator_host {
        None => {
            // do nothing
        }
        Some(firestore_emulator_host) => {
            tracing::info!(
                "FIRESTORE_EMULATOR_HOST is set to {}, using Firestore emulator",
                firestore_emulator_host
            );
        }
    }
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
    let google_user_map_repository: Arc<dyn GoogleUserMapRepository> =
        Arc::new(FirestoreGoogleUserMapRepository::new(firestore.clone()));

    // Create projections with Arc<dyn T>
    let projection = FirestoreProjection::new(firestore.clone());
    let account_projection: Arc<dyn AccountProjection> = Arc::new(projection.clone());
    let category_projection: Arc<dyn CategoryProjection> = Arc::new(projection.clone());
    let monthly_summary_projection: Arc<dyn MonthlySummaryProjection> =
        Arc::new(projection.clone());
    let transaction_projection: Arc<dyn TransactionProjection> = Arc::new(projection);

    // Create session token creator and verifier (旧 Bearer 認証用、Step 8/9 で削除予定)
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

    // OIDC client を起動時に discover
    let oidc_client = GoogleOidcClient::discover(
        &env.oidc_issuer_url,
        &env.oidc_client_id,
        &env.oidc_client_secret,
        &env.oidc_redirect_uri,
    )
    .await
    .expect("Failed to discover OIDC provider metadata");
    let oidc_client: Arc<dyn application::OidcClient> = Arc::new(oidc_client);

    // Cookie 関連の値を構築
    let cookie_key_bytes =
        hex::decode(&env.cookie_signing_secret).expect("COOKIE_SIGNING_SECRET must be valid hex");
    let cookie_key = CookieKey::from_bytes(&cookie_key_bytes);
    let base_path = BasePath(env.base_path.clone());
    let is_prod = IsProd(env.is_prod);

    // Google サインイン / サインアップ use case
    let sign_in_with_google = SignInWithGoogleUseCase::new(google_user_map_repository.clone());
    let sign_up_with_google =
        SignUpWithGoogleUseCase::new(google_user_map_repository, user_repository.clone());

    // Create application state
    let state = AppState::new(
        account_repository,
        account_projection,
        category_projection,
        monthly_summary_projection,
        transaction_projection,
        device_repository,
        creator,
        verifier,
        user_repository,
        base_path.clone(),
        cookie_key.clone(),
        is_prod,
    );

    // Create auth state
    let auth_state = AuthState::new(
        oidc_client,
        sign_in_with_google,
        sign_up_with_google,
        cookie_key,
        base_path,
        is_prod,
    );

    tracing::info!("アプリケーションの初期化が完了しました");

    // Run the server
    api::run(
        state,
        auth_state,
        env.port,
        env.public_dir.as_deref(),
        &env.base_path,
    )
    .await;
}
