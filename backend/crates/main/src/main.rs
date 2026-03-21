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
use infra::FirestoreClient;
use infra::FirestoreDeviceRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;
use infra::IamSessionTokenCreator;
use infra::IamSessionTokenVerifier;
use infra::PemSessionTokenCreator;
use infra::PemSessionTokenVerifier;
use infra::ServiceAccountCredentials;

#[tokio::main]
async fn main() {
    let env = Env::from_env().expect("Failed to load environment variables");

    println!("{:?}", env);

    // Firestore uses FIRESTORE_EMULATOR_HOST if set
    let firestore = Firestore::new(FirestoreOptions {
        project_id: Some(env.project_id.clone()),
    })
    .expect("Failed to initialize Firestore");

    let firestore_client = match env.firestore_emulator_host {
        None => {
            FirestoreClient::connect(
                infra::DatabaseName::from_project_id(env.project_id)
                    .expect("Failed to parse project_id as DatabaseName"),
            )
            .await
            .expect("Failed to connect to Firestore")
        }
        Some(_emulator_host) => FirestoreClient::connect_with_emulator(
            infra::DatabaseName::from_project_id(env.project_id)
                .expect("Failed to parse project_id as DatabaseName"),
        )
        .await
        .expect("Failed to connect to Firestore emulator"),
    };

    // Create repositories with Arc<dyn T>
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(firestore.clone()));
    let device_repository: Arc<dyn DeviceRepository> =
        Arc::new(FirestoreDeviceRepository::new(firestore_client.clone()));
    let user_repository: Arc<dyn UserRepository> =
        Arc::new(FirestoreUserRepository::new(firestore.clone()));

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
    api::run(state, env.port, env.public_dir.as_deref(), &env.base_path).await;
}
