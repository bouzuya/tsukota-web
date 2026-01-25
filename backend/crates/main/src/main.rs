use std::sync::Arc;

use api::AppState;
use api::ServiceAccountCredentials;
use api::Signer;
use application::TokenSigner;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::DeviceRepository;
use application::repository::UserRepository;
use infra::FirestoreAccountRepository;
use infra::FirestoreClient;
use infra::FirestoreDeviceRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;

#[tokio::main]
async fn main() {
    // Initialize Firestore client
    let client = FirestoreClient::connect_with_emulator().await.unwrap();

    // Create repositories with Arc<dyn T>
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(client.clone()));
    let device_repository: Arc<dyn DeviceRepository> =
        Arc::new(FirestoreDeviceRepository::new(client.clone()));
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(FirestoreUserRepository::new(client.clone()));

    // Create projections with Arc<dyn T>
    let projection = FirestoreProjection::new(client);
    let account_projection: Arc<dyn AccountProjection> = Arc::new(projection.clone());
    let category_projection: Arc<dyn CategoryProjection> = Arc::new(projection.clone());
    let transaction_projection: Arc<dyn TransactionProjection> = Arc::new(projection);

    // Create token signer
    let signer: Arc<dyn TokenSigner> = match ServiceAccountCredentials::load() {
        Ok(Some(credentials)) => {
            let signer = Signer::new(&credentials.private_key, credentials.client_email)
                .expect("Failed to create signer");
            Arc::new(signer)
        }
        Ok(None) => {
            eprintln!(
                "WARNING: GOOGLE_APPLICATION_CREDENTIALS not set. \
                 create_custom_token endpoint will not work."
            );
            // ダミー signer を作成（本番では使用不可）
            Arc::new(DummySigner)
        }
        Err(e) => {
            eprintln!("ERROR: Failed to load credentials: {}", e);
            std::process::exit(1);
        }
    };

    // Create application state
    let state = AppState::new(
        account_repository,
        account_projection,
        category_projection,
        transaction_projection,
        device_repository,
        signer,
        user_repository,
    );

    // Run the server
    api::run(state).await;
}

/// ダミー TokenSigner（認証情報がない場合のフォールバック）
struct DummySigner;

impl TokenSigner for DummySigner {
    fn now(&self) -> Result<u64, application::token_signer::SignerError> {
        Err(Box::new(std::io::Error::other("Signer not configured")))
    }

    fn sign(
        &self,
        _uid: &str,
        _now: u64,
    ) -> Result<String, application::token_signer::SignerError> {
        Err(Box::new(std::io::Error::other("Signer not configured")))
    }
}
