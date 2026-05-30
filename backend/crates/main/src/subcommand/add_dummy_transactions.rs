use std::sync::Arc;

use application::UserId;
use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::repository::AccountRepository;
use application::request::AddTransactionRequest;
use application::use_case::AddTransactionUseCase;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use date_time::DateTime;
use domain::AccountId;
use infra::FirestoreAccountRepository;
use infra::FirestoreProjection;

/// 指定アカウントに対して直近 2 年に分散させて `count` 件のダミー取引を投入する。
///
/// 通常の API 呼び出しと同じく `AddTransactionUseCase` を 1 件ずつ呼び出す
/// (集約再構築 → version 増加 → query doc 更新を経るので画面表示確認に
/// 必要な read model も整合した状態になる)。区分は事前登録済みの
/// ものを `FirestoreProjection` 経由で取得し、出現順にラウンドロビンで利用する。
pub(super) async fn run(account_id_str: &str, count: usize) {
    crate::init_tracing();

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
