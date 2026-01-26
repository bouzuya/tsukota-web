use std::sync::Arc;

use api::AppState;
use api::Creator;
use api::IamSessionTokenCreator;
use api::ServiceAccountCredentials;
use api::Verifier;
use application::SessionTokenCreator;
use application::SessionTokenVerifier;
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

    // Create session token creator and verifier
    let (creator, verifier): (Arc<dyn SessionTokenCreator>, Arc<dyn SessionTokenVerifier>) =
        match ServiceAccountCredentials::load() {
            Ok(Some(credentials)) => {
                let creator =
                    Creator::new(&credentials.private_key, credentials.client_email.clone())
                        .expect("Failed to create session token creator");
                let verifier = Verifier::new(&credentials.private_key, credentials.client_email)
                    .expect("Failed to create session token verifier");
                (Arc::new(creator), Arc::new(verifier))
            }
            Ok(None) => {
                println!("GOOGLE_APPLICATION_CREDENTIALS not set, using IamSessionTokenCreator");
                let creator = IamSessionTokenCreator::new("FIXME@example.com".to_owned());
                // FIXME
                let verifier = DummyVerifier;
                (Arc::new(creator), Arc::new(verifier))
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
        creator,
        verifier,
        user_repository,
    );

    // Run the server
    api::run(state).await;
}

/// ダミー SessionTokenVerifier（認証情報がない場合のフォールバック）
struct DummyVerifier;

impl SessionTokenVerifier for DummyVerifier {
    fn verify(&self, _token: &str) -> Result<String, application::session_token::VerifierError> {
        Err(Box::new(std::io::Error::other("Verifier not configured")))
    }
}
