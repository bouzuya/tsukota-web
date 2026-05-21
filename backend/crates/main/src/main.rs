use std::sync::Arc;

mod env;

use api::AppState;
use api::AuthState;
use api::BasePath;
use api::CookieKey;
use api::IsProd;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::MonthlySummaryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::GoogleUserMapRepository;
use application::repository::UserRepository;
use application::use_case::SignInWithGoogleUseCase;
use application::use_case::SignUpWithGoogleUseCase;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use env::Env;
use infra::FirestoreAccountRepository;
use infra::FirestoreGoogleUserMapRepository;
use infra::FirestoreProjection;
use infra::FirestoreUserRepository;
use infra::GoogleOidcClient;

#[tokio::main]
async fn main() {
    // サブコマンド解決。第 1 引数が既知のサブコマンド名なら専用処理を実行して終了する。
    // 未指定または未知の場合は従来通りサーバーを起動する。
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("generate-cookie-key") => {
            // 新しい署名鍵 (64 byte) を生成し hex 化して標準出力に書き出す。
            // 出力は `COOKIE_SIGNING_SECRET` にそのまま設定できる形式。
            let key = CookieKey::generate();
            println!("{}", hex::encode(key.master()));
            return;
        }
        Some(_) | None => {
            // フォールスルー: 従来通りサーバーを起動する
        }
    }

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
        database_id: None,
        project_id: Some(env.project_id.clone()),
    })
    .expect("Failed to initialize Firestore");

    // Create repositories with Arc<dyn T>
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(firestore.clone()));
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
        SignUpWithGoogleUseCase::new(google_user_map_repository, user_repository);

    // Create application state
    let state = AppState::new(
        account_repository,
        account_projection,
        category_projection,
        monthly_summary_projection,
        transaction_projection,
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
