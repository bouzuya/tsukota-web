//! `tsukota-server` の運用サブコマンド群。
//!
//! 各サブコマンドの実処理は同名のサブモジュールに分離し、ここでは clap の
//! `Subcommand` 定義とディスパッチのみを行う。

mod add_dummy_transactions;
mod add_owner;
mod backfill_monthly_summaries;
mod backfill_transactions;
mod generate_cookie_key;
mod run_server;

/// `tsukota-server` のサブコマンド。
///
/// 運用系のサブコマンドは単発処理で実行後にプロセスを終了する。サブコマンド
/// 未指定時の既定の挙動 (API サーバー起動) も `RunServer` として保持する。
#[derive(Debug, clap::Subcommand)]
pub(crate) enum Subcommand {
    /// 画面表示確認用のダミー取引データを一括投入する。
    ///
    /// 直近 2 年間に分散させた取引を AddTransactionUseCase 経由で 1 件ずつ追加する。
    /// 想定は Firestore Emulator 接続。必要な env: GOOGLE_CLOUD_PROJECT
    /// (または GCLOUD_PROJECT), GOOGLE_APPLICATION_CREDENTIALS,
    /// FIRESTORE_EMULATOR_HOST。投入先のアカウントには事前に最低 1 件の
    /// 区分 (Category) が登録されている必要がある。
    AddDummyTransactions {
        /// 投入先アカウント ID (UUID)
        #[arg(default_value = "bc6d2814-824b-4a78-baa4-6221ec4bbcf7")]
        account_id: String,
        /// 投入する取引件数
        #[arg(default_value_t = 1000)]
        count: usize,
    },
    /// 指定アカウントの owners に指定ユーザーを追加する。
    ///
    /// 通常の API と同じく AddOwnerUseCase 経由でコマンドを処理する。
    /// 必要な env: GOOGLE_CLOUD_PROJECT (または GCLOUD_PROJECT),
    /// GOOGLE_APPLICATION_CREDENTIALS, 任意で FIRESTORE_EMULATOR_HOST。
    AddOwner {
        /// 対象アカウント ID (UUID)
        account_id: String,
        /// owner に追加するユーザー ID (UUID)
        user_id: String,
        /// コマンドを実行するユーザー ID (UUID)。本番経路の認証済みユーザに相当。
        acting_user_id: String,
    },
    /// 取引クエリ用ドキュメント (accounts/{id}/transactions/{tx_id}) を
    /// events から一括再構築する。
    ///
    /// 本番反映前に 1 度だけ実行する想定。必要な env: GOOGLE_CLOUD_PROJECT
    /// (または GCLOUD_PROJECT), GOOGLE_APPLICATION_CREDENTIALS,
    /// 任意で FIRESTORE_EMULATOR_HOST。サーバ起動用の OIDC / Cookie 等の env は不要。
    BackfillTransactions,
    /// 月別サマリードキュメント (accounts/{id}/stats/monthly) を events から
    /// 一括再構築する。
    ///
    /// 集計が欠損・破損した場合や、`BackfillTransactions` と同様に既存アカウントの
    /// read model を初期化する際に実行する。active な取引から再集計するため
    /// Idempotent。必要な env は `BackfillTransactions` と同じ。
    BackfillMonthlySummaries,
    /// Cookie 署名鍵 (64 byte) を生成し hex 化して標準出力に書き出す。
    ///
    /// 出力は `COOKIE_SIGNING_SECRET` にそのまま設定できる形式。
    GenerateCookieKey,
    /// API サーバーを起動する。サブコマンド未指定時の既定の挙動。
    RunServer,
}

impl Subcommand {
    /// サブコマンドを実行する。
    pub(crate) async fn run(self) {
        match self {
            Subcommand::AddDummyTransactions { account_id, count } => {
                add_dummy_transactions::run(&account_id, count).await
            }
            Subcommand::AddOwner {
                account_id,
                user_id,
                acting_user_id,
            } => add_owner::run(&account_id, &user_id, &acting_user_id).await,
            Subcommand::BackfillMonthlySummaries => backfill_monthly_summaries::run().await,
            Subcommand::BackfillTransactions => backfill_transactions::run().await,
            Subcommand::GenerateCookieKey => generate_cookie_key::run(),
            Subcommand::RunServer => run_server::run().await,
        }
    }
}
