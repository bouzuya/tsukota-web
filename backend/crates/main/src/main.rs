mod env;
mod subcommand;

use clap::Parser;
use subcommand::Subcommand;

/// `tsukota-server` バイナリの CLI 定義。
///
/// サブコマンド未指定時は既定の挙動 (`RunServer`) として API サーバーを起動する。
/// 各サブコマンドは `subcommand` モジュールに実装する。
#[derive(Debug, Parser)]
#[command(name = "tsukota-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Subcommand>,
}

/// tracing subscriber を環境変数 (`RUST_LOG` 等) ベースで初期化する。
/// 未設定時は `info` レベルで出力する。
pub(crate) fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

#[tokio::main]
async fn main() {
    // サブコマンド未指定時は既定の挙動 (API サーバー起動) に fallback する。
    Cli::parse()
        .command
        .unwrap_or(Subcommand::RunServer)
        .run()
        .await;
}
