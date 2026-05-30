use std::sync::Arc;

mod env;

use api::AppState;
use api::AuthState;
use api::BasePath;
use api::CookieKey;
use api::IsProd;
use application::UserId;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::MonthlySummaryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::GoogleUserMapRepository;
use application::repository::UserRepository;
use application::request::AddTransactionRequest;
use application::use_case::AddTransactionUseCase;
use application::use_case::SignInWithGoogleUseCase;
use application::use_case::SignUpWithGoogleUseCase;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use date_time::DateTime;
use domain::AccountId;
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
        Some("add-dummy-transactions") => {
            // 画面表示確認用のダミー取引データを大量投入する。
            // 直近 2 年間に分散させて 1000 件の取引を、AddTransactionUseCase 経由で 1 件ずつ追加する。
            //
            // 想定: Firestore Emulator 接続。
            // 必要な env: GOOGLE_CLOUD_PROJECT (または GCLOUD_PROJECT),
            //   GOOGLE_APPLICATION_CREDENTIALS, FIRESTORE_EMULATOR_HOST。
            // 投入先のアカウントには事前に最低 1 件の区分 (Category) が登録されている必要がある。
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .init();

            // 引数: [account_id] [count]
            // どちらも省略可。account_id 未指定時はハードコード値、count 未指定時は 1000。
            let account_id_str = args
                .next()
                .unwrap_or_else(|| "bc6d2814-824b-4a78-baa4-6221ec4bbcf7".to_string());
            let count: usize = args
                .next()
                .map(|s| s.parse().expect("count must be a positive integer"))
                .unwrap_or(1000);

            add_dummy_transactions(&account_id_str, count).await;
            return;
        }
        Some("backfill-transactions") => {
            // 取引クエリ用ドキュメント (accounts/{id}/transactions/{tx_id}) を
            // events から一括再構築する。本番反映前に 1 度だけ実行する想定。
            //
            // 必要な env: GOOGLE_CLOUD_PROJECT (または GCLOUD_PROJECT),
            //   GOOGLE_APPLICATION_CREDENTIALS, 任意で FIRESTORE_EMULATOR_HOST。
            // サーバ起動用の OIDC / Cookie 等の env は不要。
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                )
                .init();

            // project_id は Firestore client が GOOGLE_CLOUD_PROJECT / GCLOUD_PROJECT から
            // 自動検出する。
            let firestore = Firestore::new(FirestoreOptions::default())
                .expect("Failed to initialize Firestore");
            let account_repository = FirestoreAccountRepository::new(firestore.clone());

            let stats =
                infra::backfill::backfill_query_transactions(&firestore, &account_repository)
                    .await
                    .expect("backfill failed");
            tracing::info!(?stats, "backfill complete");
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

/// 指定アカウントに対して直近 2 年に分散させて `count` 件のダミー取引を投入する。
///
/// 通常の API 呼び出しと同じく `AddTransactionUseCase` を 1 件ずつ呼び出す
/// (集約再構築 → version 増加 → query doc 更新を経るので画面表示確認に
/// 必要な read model も整合した状態になる)。区分は事前登録済みの
/// ものを `FirestoreProjection` 経由で取得し、出現順にラウンドロビンで利用する。
async fn add_dummy_transactions(account_id_str: &str, count: usize) {
    use std::sync::Arc;

    use application::projection::AccountProjection as _;
    use application::projection::CategoryProjection;

    let account_id: AccountId = account_id_str
        .parse()
        .expect("Invalid account id (expected UUID)");

    // Firestore 初期化 (backfill サブコマンドと同じく default options で
    // GOOGLE_CLOUD_PROJECT / FIRESTORE_EMULATOR_HOST から接続情報を解決する)
    let firestore =
        Firestore::new(FirestoreOptions::default()).expect("Failed to initialize Firestore");

    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(firestore.clone()));
    let projection = FirestoreProjection::new(firestore);

    // 既存の区分を読み込む。1 件もなければダミーデータ投入は不可能なので即終了する。
    let categories = projection
        .list_categories(&account_id)
        .await
        .expect("Failed to list categories");
    if categories.is_empty() {
        panic!(
            "No categories registered for account {}; create at least one category first.",
            account_id_str
        );
    }

    // アカウントの owners を取得し、user_id として先頭の owner を使う。
    // 通常 API では認証済みユーザの user_id が渡されるので、ダミー UUID を使わず
    // 既存 owner を割り当てることで本番経路となるべく挙動を揃える。
    let owner_ids = projection
        .list_account_owner_ids(&account_id)
        .await
        .expect("Failed to list account owners");
    let owner_str = owner_ids.first().unwrap_or_else(|| {
        panic!(
            "Account {} has no owners; cannot pick a user id for dummy transactions.",
            account_id_str
        )
    });
    let acting_user_id: UserId = owner_str
        .parse()
        .expect("Account owner id is not a valid UserId");

    tracing::info!(
        account_id = %account_id_str,
        category_count = categories.len(),
        owner_count = owner_ids.len(),
        acting_user_id = %acting_user_id,
        total = count,
        "add-dummy-transactions start"
    );

    let use_case = AddTransactionUseCase::new(account_repository);

    // 直近 2 年 (24 ヶ月) に分散する日付列を生成する。
    // 28 までしか使わないので閏年・月末問題は発生しない。
    let dates = generate_dates_last_two_years(count);

    let mut success = 0usize;
    for (i, date) in dates.iter().enumerate() {
        let category = &categories[i % categories.len()];
        // 偶数番目は収入、奇数番目は支出のダミー金額。
        // 月別集計画面で +/- 両方の集計が出るようにする。
        let amount = if i.is_multiple_of(2) {
            format!("{}", 1000 + (i as i64 % 9000))
        } else {
            format!("-{}", 500 + (i as i64 % 4500))
        };
        let request = AddTransactionRequest {
            account_id: account_id_str.to_string(),
            amount,
            category_id: category.id.clone(),
            comment: format!("seed #{:04}", i + 1),
            date: date.clone(),
        };

        match use_case.execute(&acting_user_id, request).await {
            Ok(_) => {
                success += 1;
                if success.is_multiple_of(100) {
                    tracing::info!(
                        progress = success,
                        total = count,
                        "add-dummy-transactions progress"
                    );
                }
            }
            Err(e) => {
                tracing::error!(index = i, error = ?e, "Failed to add transaction");
                // 1 件失敗しても継続する (Firestore の一時エラーなどで途中停止すると
                // 再開が面倒なため)。
            }
        }
    }

    tracing::info!(success, total = count, "add-dummy-transactions complete");
}

/// 「今日」を起点に過去 2 年 (24 ヶ月) へ均等に散らした `count` 件分の
/// "YYYY-MM-DD" 文字列を生成する。
///
/// - 月数は (i / count * 24) で割り当て (現在月 = 0, 過去ほど大きい)
/// - 日は (i % 28) + 1 (どの月でも有効)
///
/// 入力順は古い → 新しい (画面表示で時系列が前向きに増えるイメージ)
fn generate_dates_last_two_years(count: usize) -> Vec<String> {
    let (today_year, today_month) = current_year_month();
    let mut dates = Vec::with_capacity(count);
    // i = 0 を最古、i = count - 1 を最新にする。
    for i in 0..count {
        // 0..=23 の月オフセット (0 が現在月)
        let months_back = if count <= 1 {
            0
        } else {
            // 古い側から並べるので反転する
            let from_oldest = count - 1 - i;
            // count 件を 24 ヶ月に均等分布
            (from_oldest as u32 * 23) / (count as u32 - 1)
        };
        let (y, m) = subtract_months(today_year, today_month, months_back);
        let d = (i % 28) as u32 + 1;
        dates.push(format!("{:04}-{:02}-{:02}", y, m, d));
    }
    dates
}

/// `date-time` crate の `DateTime::now()` から年月を取り出す。
/// RFC3339 ("YYYY-MM-DDT..." 形式) の先頭 7 文字をそのままパースする。
fn current_year_month() -> (i32, u32) {
    let now = DateTime::now().to_string();
    // RFC3339 仕様により "YYYY-MM-DD..." の固定先頭を持つ
    let year: i32 = now[0..4].parse().expect("invalid year in DateTime::now");
    let month: u32 = now[5..7].parse().expect("invalid month in DateTime::now");
    (year, month)
}

/// `(year, month)` から `months_back` ヶ月前の `(year, month)` を返す。
/// month は 1..=12。
fn subtract_months(year: i32, month: u32, months_back: u32) -> (i32, u32) {
    // 0-indexed の通算月で計算する
    let zero_indexed = (year as i64) * 12 + (month as i64) - 1;
    let target = zero_indexed - months_back as i64;
    let new_year = target.div_euclid(12) as i32;
    let new_month = (target.rem_euclid(12) + 1) as u32;
    (new_year, new_month)
}
