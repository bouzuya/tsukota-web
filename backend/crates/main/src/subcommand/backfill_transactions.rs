use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use infra::FirestoreAccountRepository;

/// 取引クエリ用ドキュメント (accounts/{id}/transactions/{tx_id}) を
/// events から一括再構築する。本番反映前に 1 度だけ実行する想定。
pub(super) async fn run() {
    crate::init_tracing();

    // project_id は Firestore client が GOOGLE_CLOUD_PROJECT / GCLOUD_PROJECT から
    // 自動検出する。
    let firestore =
        Firestore::new(FirestoreOptions::default()).expect("Failed to initialize Firestore");
    let account_repository = FirestoreAccountRepository::new(firestore.clone());

    let stats = infra::backfill::backfill_query_transactions(&firestore, &account_repository)
        .await
        .expect("backfill failed");
    tracing::info!(?stats, "backfill complete");
}
