# Plan: api crate のモジュール整理

## 概要

`backend/crates/api` の SessionToken 関連の実装を 1 つのモジュールに統合する。

## 現状分析

### 現在のファイル構造

```
backend/crates/api/src/
├── lib.rs              # モジュール定義と re-export
├── signer.rs           # ローカル開発用: Creator, Verifier
├── iam_signer.rs       # Cloud Run 用: IamSessionTokenCreator, IamSessionTokenVerifier
├── session_claims.rs   # JWT クレーム: SessionTokenClaims
├── credentials.rs      # サービスアカウント認証情報: ServiceAccountCredentials
├── extractor.rs        # Axum 認証抽出器
├── state.rs            # アプリケーション状態
├── error.rs            # API エラー
├── router.rs           # ルーター
└── handler/
    ├── mod.rs
    ├── command.rs
    └── query.rs
```

### SessionToken 関連の型

| ファイル | 型 | 役割 |
|---------|-----|------|
| `session_claims.rs` | `SessionTokenClaims` | JWT ペイロード定義 |
| `signer.rs` | `Creator` | ローカル用トークン作成 |
| `signer.rs` | `Verifier` | ローカル用トークン検証 |
| `signer.rs` | `CreateError`, `VerifyError` | エラー型 |
| `iam_signer.rs` | `IamSessionTokenCreator` | Cloud Run 用トークン作成 |
| `iam_signer.rs` | `IamSessionTokenVerifier` | Cloud Run 用トークン検証 |
| `iam_signer.rs` | `IamSessionTokenCreateError`, `IamSessionTokenVerifyError` | エラー型 |
| `credentials.rs` | `ServiceAccountCredentials` | 認証情報読み込み |
| `credentials.rs` | `CredentialsError` | エラー型 |

## 実装計画

### 統合後のファイル構造

```
backend/crates/api/src/
├── lib.rs
├── session_token.rs         # 新規: モジュール定義と re-export
├── session_token/           # 新規: SessionToken 関連を統合
│   ├── claims.rs           # SessionTokenClaims
│   ├── credentials.rs      # ServiceAccountCredentials
│   ├── local.rs            # Creator, Verifier (ローカル開発用)
│   └── iam.rs              # IamSessionTokenCreator, IamSessionTokenVerifier (Cloud Run 用)
├── extractor.rs
├── state.rs
├── error.rs
├── router.rs
└── handler.rs
    └── handler/
        ├── command.rs
        └── query.rs
```

### 変更手順

1. **`session_token/` ディレクトリを作成**

2. **ファイルを移動・リネーム**
   - `session_claims.rs` → `session_token/claims.rs`
   - `credentials.rs` → `session_token/credentials.rs`
   - `signer.rs` → `session_token/local.rs`
   - `iam_signer.rs` → `session_token/iam.rs`

3. **`session_token.rs` を作成**
   ```rust
   mod claims;
   mod credentials;
   mod iam;
   mod local;

   pub use self::claims::SessionTokenClaims;
   pub use self::credentials::CredentialsError;
   pub use self::credentials::ServiceAccountCredentials;
   pub use self::iam::IamSessionTokenCreateError;
   pub use self::iam::IamSessionTokenCreator;
   pub use self::iam::IamSessionTokenVerifier;
   pub use self::iam::IamSessionTokenVerifyError;
   pub use self::local::CreateError;
   pub use self::local::Creator;
   pub use self::local::Verifier;
   pub use self::local::VerifyError;
   ```

4. **`lib.rs` を更新**
   ```rust
   mod error;
   mod extractor;
   mod handler;
   mod router;
   mod session_token;  // 新規
   mod state;

   pub use self::session_token::CredentialsError;
   pub use self::session_token::ServiceAccountCredentials;
   // ... その他の re-export
   ```

5. **内部参照を更新**
   - `local.rs` 内の `use crate::session_claims::SessionTokenClaims;`
     → `use super::claims::SessionTokenClaims;`
   - `iam.rs` 内の同様の参照を更新

## 変更対象ファイル

1. **新規作成**
   - `backend/crates/api/src/session_token.rs`

2. **移動（内容修正あり）**
   - `session_claims.rs` → `session_token/claims.rs`
   - `credentials.rs` → `session_token/credentials.rs`
   - `signer.rs` → `session_token/local.rs`
   - `iam_signer.rs` → `session_token/iam.rs`

3. **修正**
   - `backend/crates/api/src/lib.rs`

4. **削除**
   - `session_claims.rs`, `credentials.rs`, `signer.rs`, `iam_signer.rs`（移動後）

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
