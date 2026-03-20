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

1. **日本語でコメント・ドキュメントを書く**
2. **Rust**: `mod.rs` を使わない、`BTreeMap`/`BTreeSet` を使う
3. **テスト**: `anyhow::Result<()>` を返す、`unwrap()` を避ける
4. **イベントソーシング**: tsukota との互換性を維持する

## プランの保存（Claude Code 用）

- 後で振り返ることを目的としてプランモードの結果を保存する
- 保存先はプロンプト `prompts/{N}.md` に対してのプランを `prompts/{N}-plan.md` とする
