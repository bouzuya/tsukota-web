use std::sync::Arc;

use application::UserId;
use application::repository::AccountRepository;
use application::request::AddOwnerRequest;
use application::use_case::AddOwnerUseCase;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::FirestoreOptions;
use domain::AccountId;
use infra::FirestoreAccountRepository;

/// 指定アカウントの owners に指定ユーザーを追加する。
///
/// 通常の API 呼び出しと同じく `AddOwnerUseCase` を経由する
/// (集約再構築 → コマンド処理 → events 永続化 → query doc 更新)。
/// これにより read model も整合した状態になる。
///
/// `acting_user_id` はコマンドを実行するユーザー (本番経路の認証済みユーザに相当)、
/// `user_id_str` は owner に追加する対象ユーザー。
pub(super) async fn run(account_id_str: &str, user_id_str: &str, acting_user_id_str: &str) {
    crate::init_tracing();

    // 引数の妥当性を実行前に検証し、不正な場合は早期終了する。
    let account_id: AccountId = account_id_str
        .parse()
        .expect("Invalid account id (expected UUID)");
    let acting_user_id: UserId = acting_user_id_str
        .parse()
        .expect("Invalid acting user id (expected UUID)");

    // Firestore 初期化 (他サブコマンドと同じく default options で
    // GOOGLE_CLOUD_PROJECT / FIRESTORE_EMULATOR_HOST から接続情報を解決する)
    let firestore =
        Firestore::new(FirestoreOptions::default()).expect("Failed to initialize Firestore");

    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(FirestoreAccountRepository::new(firestore));

    tracing::info!(
        account_id = %account_id,
        user_id = %user_id_str,
        acting_user_id = %acting_user_id,
        "add-owner start"
    );

    let use_case = AddOwnerUseCase::new(account_repository);
    let request = AddOwnerRequest {
        account_id: account_id_str.to_string(),
        user_id: user_id_str.to_string(),
    };

    use_case
        .execute(&acting_user_id, request)
        .await
        .expect("Failed to add owner");

    tracing::info!(
        account_id = %account_id,
        user_id = %user_id_str,
        acting_user_id = %acting_user_id,
        "add-owner complete"
    );
}
