# AGENTS.md

このファイルは AI エージェント（Claude Code, GitHub Copilot CLI など）にプロジェクトのコンテキストを提供します。

## プロジェクト概要

tsukota-web は家計簿アプリ [tsukota](https://github.com/bouzuya/tsukota) の Web 版です。

- **Backend**: Rust (axum, イベントソーシング)
- **Frontend**: React + TypeScript (Vite, Tailwind CSS)
- **Database**: Firestore

## ドキュメント

以下のドキュメントを必ず参照してください：

- [docs/CODING_GUIDELINES.md](docs/CODING_GUIDELINES.md) - コーディングスタイルとルール
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - アーキテクチャ設計
- [docs/SPEC.md](docs/SPEC.md) - API 仕様

## よく使うコマンド

### Backend (Rust)

```bash
# ビルド
cargo build

# テスト
cargo test

# フォーマット（nightly 必須）
cargo +nightly fmt

# Lint
cargo clippy

# テストカバレッジ計測（cargo-llvm-cov を使用）
cargo llvm-cov --workspace

# ドキュメント生成
cargo doc --open
```

### Frontend (TypeScript/React)

```bash
cd frontend

# 依存関係インストール
npm install

# 開発サーバー起動
npm run dev

# ビルド
npm run build

# Lint & Format
npm run lint
npm run check
```

### CLI サブコマンド (backend)

`tsukota-server` バイナリは第 1 引数で運用サブコマンドを受け付けます。未指定時は API サーバーとして起動します。

```bash
# Cookie 署名鍵を生成 (COOKIE_SIGNING_SECRET に設定する hex 文字列)
cargo run -p main -- generate-cookie-key

# 取引クエリ用ドキュメント (accounts/{id}/transactions/{tx_id}) を events から
# 一括再構築する。永続化 read model 導入後に既存アカウントを初期化するため 1 度だけ実行。
# 必要 env: GOOGLE_CLOUD_PROJECT (または GCLOUD_PROJECT) と
#           GOOGLE_APPLICATION_CREDENTIALS。任意で FIRESTORE_EMULATOR_HOST。
# OIDC / Cookie 関連の env は不要。Idempotent (再実行しても結果は同じ)。
cargo run -p main -- backfill-transactions

# 月別サマリードキュメント (accounts/{id}/stats/monthly) を events から一括再構築する。
# 集計が欠損・破損した場合や、backfill-transactions と同様に既存アカウントの
# read model を初期化する際に実行する。現在 active な取引から再集計するので
# Idempotent。必要 env は backfill-transactions と同じ。
cargo run -p main -- backfill-monthly-summaries

# 画面表示の確認用に、指定アカウントへ直近 2 年分のダミー取引を一括投入する。
# AddTransactionUseCase を 1 件ずつ呼ぶので、本番経路と同じ集約再構築・query
# ドキュメント更新を経る。事前に最低 1 件の区分 (Category) を登録しておくこと。
# 引数: [account_id] [count]。省略時は account_id=bc6d2814-... / count=1000。
# 必要 env: backfill-transactions と同じ (Firestore Emulator 想定)。
cargo run -p main -- add-dummy-transactions
cargo run -p main -- add-dummy-transactions <account-id> 500
```

## プロジェクト構造

```
backend/
├── crates/
│   ├── domain/       # ドメイン層（エンティティ、イベント）
│   ├── application/  # アプリケーション層（ユースケース）
│   ├── infra/        # インフラ層（Firestore 実装）
│   ├── api/          # プレゼンテーション層（axum ハンドラー）
│   └── main/         # エントリーポイント

frontend/
├── src/
│   ├── api/          # API クライアント
│   ├── atoms/        # Jotai atoms（状態管理）
│   ├── components/   # 再利用可能コンポーネント
│   ├── hooks/        # カスタムフック
│   └── pages/        # ページコンポーネント
```

## コーディングガイドライン

### Rust

- `mod.rs` を使用しない（`account/mod.rs` ではなく `account.rs`）
- `HashMap`/`HashSet` ではなく `BTreeMap`/`BTreeSet` を使用
- `if let` ではなく `match` を使用し、`unreachable!()` で到達不可を明示
- derive マクロは alphabetical order で記述
- インポートは `imports_granularity = "Item"` に従い個別に記述
- テストは `anyhow::Result<()>` を返す（`unwrap()` を避ける）
- コメントは日本語で記述
- エラーの `Display` 表現（`thiserror` の `#[error("...")]` を含む）は
  [Rust API Guidelines C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)
  に従い、**小文字始まり・末尾の句読点なし・簡潔** に書く。
  例: `"invalid account id"` (✓) / `"Invalid account ID."` (✗)。
  これは他の文字列に埋め込まれて表示されることが多いため。
- **module local なエラー定義**: ファイル内に閉じた処理経路のエラーは
  そのファイル内に `enum E` (短名で module local であることを強調) として
  `thiserror::Error` derive 付きで定義する。`pub` にしない。
  - `map_err` でどの処理に由来するエラーかが分かるよう、**処理単位で
    variant を切る** (例: `LoadEvents` / `SaveEvents` / `LoadUserEvents`)。
  - 元エラーは `#[source]` で保持し、source chain を切らない。
  - 公開エラー型 (`ApplicationError` 等) への変換は
    `impl From<E> for ApplicationError` 1 箇所で行う。呼び出し側は
    `.map_err(E::Variant)?` だけで `?` 経由で上位エラーに変換される。
  - variant も derive と同様に **アルファベット順** で並べる。
  - 参考実装: `backend/crates/infra/src/firestore_projection.rs` /
    `backend/crates/infra/src/firestore_account_repository.rs`。

### TypeScript/React

- Biome でフォーマット・リント
- Jotai で状態管理
- Tailwind CSS でスタイリング

### 命名規則

- **イベント名**: 過去形 (`AccountCreated`, `OwnerAdded`)
- **コマンド名**: 動詞 + 名詞 (`CreateAccount`, `AddOwner`)
- **ID 型**: newtype パターンで Value Object として定義

## アーキテクチャ

```
api → application → domain
        ↓
      infra → domain
```

- `domain` は他のレイヤーに依存しない
- `infra` は `domain` の trait を実装
- イベントソーシングで状態管理（集約単位は Account のみ）

## 重要なルール

- 日本語でコメント・ドキュメントを書く、ただしコミットログは英語で書く
- **テスト**: `anyhow::Result<()>` を返す、`unwrap()` を避ける
