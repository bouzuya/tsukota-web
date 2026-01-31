# tsukota-web

[tsukota](https://github.com/bouzuya/tsukota) の Web 版実装

複数ユーザーで共有可能なアカウント・支出管理 Web アプリケーションです。

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フロントエンド | React + Vite + Tailwind CSS |
| バックエンド | Rust + axum |
| データベース | Firestore |
| 認証 | デバイス認証 + JWT |
| デプロイ | Cloud Run |

## プロジェクト構成

```
tsukota-web/
├── backend/          # Rust バックエンド
│   └── crates/
│       ├── domain/       # ドメイン層
│       ├── application/  # アプリケーション層
│       ├── infra/        # インフラ層
│       ├── api/          # プレゼンテーション層
│       └── main/         # エントリーポイント
├── frontend/         # React フロントエンド
├── firebase/         # Firebase Emulator 設定
└── docs/             # ドキュメント
```

## 開発環境のセットアップ

### 前提条件

- Docker / Docker Compose
- Rust (nightly)
- Node.js

### ローカル開発

1. VS Code で Dev Container を起動 (workspace のほか Firebase Emulator も起動される)


2. バックエンドを起動:

```bash
cd backend
cargo run
```

3. フロントエンドを起動:

```bash
cd frontend
npm install
npm run dev
```

## コマンド

### Backend (Rust)

```bash
cd backend

# ビルド
cargo build

# テスト
cargo test

# フォーマット (nightly 必須)
cargo +nightly fmt

# Lint
cargo clippy
```

### Frontend (TypeScript/React)

```bash
cd frontend

# 開発サーバー
npm run dev

# ビルド
npm run build

# Lint & Format
npm run lint
npm run check
```

## ドキュメント

- [仕様書 (SPEC.md)](docs/SPEC.md) - 機能要件、API 仕様、画面構成
- [アーキテクチャ (ARCHITECTURE.md)](docs/ARCHITECTURE.md) - 技術設計、イベントソーシング
- [コーディングガイドライン (CODING_GUIDELINES.md)](docs/CODING_GUIDELINES.md) - コードスタイル

## AI の使用について

このプロジェクトは [Claude Code](https://claude.ai/claude-code) および [GitHub Copilot CLI](https://github.com/features/copilot/cli/) を使用して開発されています。

## ライセンス

MIT
