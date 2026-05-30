use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use infra::FirestoreAccountRepository;

/// 月別サマリードキュメント (accounts/{id}/stats/monthly) を events から
/// 一括再構築する。集計の欠損・破損時や既存アカウントの read model 初期化に使う。
pub(super) async fn run() {
    crate::init_tracing();

    // project_id は Firestore client が GOOGLE_CLOUD_PROJECT / GCLOUD_PROJECT から
    // 自動検出する。
    let firestore =
        Firestore::new(FirestoreOptions::default()).expect("Failed to initialize Firestore");
    let account_repository = FirestoreAccountRepository::new(firestore.clone());

    let stats = infra::backfill::backfill_monthly_summaries(&firestore, &account_repository)
        .await
        .expect("backfill failed");
    tracing::info!(?stats, "backfill complete");
}
