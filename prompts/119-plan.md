# Plan: lib.rs から実装を切り出してモジュール化

## 概要

3 つの crate の lib.rs から実装コードを別モジュールに切り出し、lib.rs を `mod` と `pub use` のみにする。

## 対象ファイル

1. `backend/crates/api/src/lib.rs` - `run()` 関数
2. `backend/crates/date-time/src/lib.rs` - `DateTime`, `ParseDateTimeError`
3. `backend/crates/firestore-client/src/lib.rs` - `FirestoreClient`, `Transaction`, `Error`

## 変更計画

### 1. api crate

**現状**: `run()` 関数が lib.rs に定義されている

**変更後**:
- `run.rs` を新規作成し `run()` 関数を移動
- lib.rs は mod/pub use のみ

```
backend/crates/api/src/
├── lib.rs          # mod run; pub use self::run::run;
└── run.rs          # run() 関数
```

### 2. date-time crate

**現状**: `DateTime`, `ParseDateTimeError`, impl, tests が lib.rs に定義

**変更後**:
- `date_time.rs` を新規作成し全て移動
- lib.rs は mod/pub use のみ

```
backend/crates/date-time/src/
├── lib.rs          # mod date_time; pub use self::date_time::{DateTime, ParseDateTimeError};
└── date_time.rs    # DateTime, ParseDateTimeError, impl, tests
```

### 3. firestore-client crate

**現状**: `FirestoreClient`, `Transaction`, `Error`, エラー enum `E`, impl, tests が lib.rs に定義

**変更後**:
- `firestore_client.rs` を新規作成し全て移動
- lib.rs は mod/pub use のみ

```
backend/crates/firestore-client/src/
├── lib.rs                # mod firestore_client; pub use ...;
└── firestore_client.rs   # FirestoreClient, Transaction, Error, E, impl, tests
```

## 変更対象ファイル一覧

### 新規作成
- `backend/crates/api/src/run.rs`
- `backend/crates/date-time/src/date_time.rs`
- `backend/crates/firestore-client/src/firestore_client.rs`

### 修正
- `backend/crates/api/src/lib.rs`
- `backend/crates/date-time/src/lib.rs`
- `backend/crates/firestore-client/src/lib.rs`

## 検証手順

```bash
cd /workspaces/tsukota-web/backend

# フォーマット
cargo +nightly fmt

# ビルド
cargo build

# テスト
cargo test

# Lint
cargo clippy
```
