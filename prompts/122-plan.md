# Plan: handler モジュールを 1 handler 1 mod に分割

## 概要

`backend/crates/api/src/handler.rs` に統合されている 17 個のハンドラーを、それぞれ個別のファイルに分割する。

## 現在の構造

```
backend/crates/api/src/
└── handler.rs              # 17 個のハンドラーがすべて含まれる
```

## 変更後の構造

```
backend/crates/api/src/
├── handler.rs              # mod 宣言と pub use のみ
└── handler/
    ├── add_category.rs
    ├── add_owner.rs
    ├── add_transaction.rs
    ├── create_account.rs
    ├── create_session_token.rs
    ├── delete_account.rs
    ├── delete_category.rs
    ├── delete_transaction.rs
    ├── export_transactions.rs
    ├── get_account.rs
    ├── list_accounts.rs
    ├── list_categories.rs
    ├── list_transactions.rs
    ├── remove_owner.rs
    ├── update_account.rs
    ├── update_category.rs
    └── update_transaction.rs
```

## 各ハンドラーファイルの内容

各ファイルには以下を含める：
- 必要なインポート文
- ハンドラー関数（pub async fn）
- 必要に応じて関連する構造体や定数

### 特記事項

- `list_transactions.rs`: `ListTransactionsParams` 構造体と `DEFAULT_PAGE_SIZE` 定数を含む
- `export_transactions.rs`: `ExportTransactionsParams` 構造体を含む

## handler.rs の内容

```rust
mod add_category;
mod add_owner;
mod add_transaction;
mod create_account;
mod create_session_token;
mod delete_account;
mod delete_category;
mod delete_transaction;
mod export_transactions;
mod get_account;
mod list_accounts;
mod list_categories;
mod list_transactions;
mod remove_owner;
mod update_account;
mod update_category;
mod update_transaction;

pub use add_category::add_category;
pub use add_owner::add_owner;
pub use add_transaction::add_transaction;
pub use create_account::create_account;
pub use create_session_token::create_session_token;
pub use delete_account::delete_account;
pub use delete_category::delete_category;
pub use delete_transaction::delete_transaction;
pub use export_transactions::export_transactions;
pub use get_account::get_account;
pub use list_accounts::list_accounts;
pub use list_categories::list_categories;
pub use list_transactions::list_transactions;
pub use remove_owner::remove_owner;
pub use update_account::update_account;
pub use update_category::update_category;
pub use update_transaction::update_transaction;
```

## router.rs への影響

変更なし。`handler::list_accounts` などの参照はそのまま動作する。

## 検証手順

1. `cargo build` でビルドが成功することを確認
2. `cargo test` でテストが通ることを確認
3. `cargo clippy` で警告がないことを確認
4. `cargo +nightly fmt` でフォーマットを適用
