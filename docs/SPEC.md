# tsukota-web 仕様書

> このドキュメントは、tsukota-web の**機能要件**と**外部仕様**を定義します。
> 「何を作るか」（機能一覧、データモデル、API仕様、画面構成）を記載しています。
>
> 技術選定やアーキテクチャ設計など「どう作るか」については [ARCHITECTURE.md](./ARCHITECTURE.md) を参照してください。

## 概要

tsukota-web は、複数ユーザーで共有可能なアカウント・支出管理 Web アプリケーションです。

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フロントエンド | React + Vite (SPA) |
| バックエンド | Rust + axum |
| データベース | Firestore |
| 認証 | Google OAuth + JWT |
| 状態管理 | jotai |
| CSS | Tailwind CSS |
| デプロイ | Cloud Run / Docker (ローカル) |

詳細なアーキテクチャ設計については [ARCHITECTURE.md](./ARCHITECTURE.md) を参照してください。

## 機能一覧

### 1. ユーザー認証

- Google OAuth によるログイン
- ログアウト機能

### 2. アカウント管理

- アカウントの作成・編集・削除
- 複数ユーザーでのアカウント共有
- 共有メンバーの招待・管理 (※初期バージョンでは保留、DB直接操作で対応)
- オーナー権限: アカウント削除、メンバー追加・削除が可能
- メンバー権限: 収支の登録・編集・削除、カテゴリ管理が可能

### 3. 収支記録

- 収入の登録・編集・削除
- 支出の登録・編集・削除
- 日付、金額、カテゴリ、メモの入力
- 対応通貨: 日本円のみ
- 編集・削除権限: 作成者以外のメンバーも可能

### 4. カテゴリ管理

- ユーザーによるカテゴリのカスタマイズ
- カテゴリの追加・編集・削除 (論理削除)
- 新規カテゴリの order: 既存カテゴリ数 + 1
- 削除されたカテゴリ: 一覧に非表示、新規取引で選択不可、既存取引の値は維持
- デフォルトカテゴリなし (ユーザーが自身で作成)

### 5. データエクスポート

- JSON 形式でのエクスポート (月別)

## データモデル

以下のデータモデルは API レスポンスおよび読み取りモデル (Projection) を表します。
アカウント、カテゴリ、取引の実体はイベントソーシングにより管理されます（詳細は [ARCHITECTURE.md](./ARCHITECTURE.md) 参照）。

### User

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | ユーザー ID (Firebase Auth UID) |
| email | string | メールアドレス |
| displayName | string | 表示名 |
| createdAt | timestamp | 作成日時 |

### Account (アカウント)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | アカウント ID |
| name | string | アカウント名 |
| ownerId | string | オーナーのユーザー ID |
| memberIds | string[] | メンバーのユーザー ID 一覧 |
| createdAt | timestamp | 作成日時 |
| updatedAt | timestamp | 更新日時 |

### Category

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | カテゴリ ID |
| accountId | string | 所属するアカウント ID |
| name | string | カテゴリ名 |
| type | string | "income" または "expense" |
| order | number | 表示順 |
| createdAt | timestamp | 作成日時 |
| deletedAt | timestamp? | 削除日時 (論理削除) |

### Transaction (収支記録)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | 取引 ID |
| accountId | string | 所属するアカウント ID |
| type | string | "income" または "expense" |
| amount | number | 金額 (円) |
| categoryId | string | カテゴリ ID |
| date | date | 取引日 |
| memo | string? | メモ |
| createdBy | string | 作成者のユーザー ID |
| createdAt | timestamp | 作成日時 |
| updatedAt | timestamp | 更新日時 |

## API エンドポイント

### 認証

- `GET /auth/google` - Google OAuth 開始
- `GET /auth/callback` - OAuth コールバック
- `POST /auth/logout` - ログアウト
- `GET /auth/me` - 現在のユーザー情報取得
- `POST /auth/refresh` - トークンリフレッシュ

### ユーザー

- `GET /users/:id` - ユーザー情報取得
- `PATCH /users/:id` - ユーザー情報更新 (displayName)

### アカウント

- `GET /accounts` - アカウント一覧取得 (オーナーまたはメンバーのアカウント)
- `POST /accounts` - アカウント作成
- `GET /accounts/:id` - アカウント詳細取得
- `PATCH /accounts/:id` - アカウント更新
- `DELETE /accounts/:id` - アカウント削除
- `POST /accounts/:id/members` - メンバー招待
- `DELETE /accounts/:id/members/:userId` - メンバー削除

### カテゴリ

- `GET /accounts/:id/categories` - カテゴリ一覧取得
- `POST /accounts/:id/categories` - カテゴリ作成
- `PATCH /accounts/:id/categories/:categoryId` - カテゴリ更新
- `DELETE /accounts/:id/categories/:categoryId` - カテゴリ削除

### 収支記録

- `GET /accounts/:id/transactions` - 取引一覧取得 (20件/ページ、日付降順、cursor ベース: `?after=<id>`)
- `POST /accounts/:id/transactions` - 取引作成
- `PATCH /accounts/:id/transactions/:transactionId` - 取引更新
- `DELETE /accounts/:id/transactions/:transactionId` - 取引削除

### エクスポート

- `GET /accounts/:id/export/json?year=YYYY&month=MM` - JSON エクスポート (月別)

## 画面一覧

1. ログイン画面
2. ダッシュボード (アカウント選択)
3. 収支一覧画面
4. 収支登録・編集画面
5. カテゴリ管理画面
6. アカウント設定画面 (メンバー管理含む)
7. ユーザー設定画面

## 非機能要件

- レスポンシブデザイン (モバイル対応)
- 日本語 UI
- セキュアな認証・認可
- API エラーレスポンス: RFC 9457 (Problem Details for HTTP APIs) 準拠
