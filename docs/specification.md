# tsukota-web 仕様書

## 概要

tsukota-web は、複数ユーザーで共有可能なアカウント・支出管理 Web アプリケーションです。

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フロントエンド | React + Vite (SPA) |
| バックエンド | Rust + axum |
| データベース | Firestore |
| 認証 | OAuth (Google 等) |
| デプロイ | Cloud Run / Docker (ローカル) |

## 機能一覧

### 1. ユーザー認証

- Google OAuth によるログイン
- ログアウト機能

### 2. アカウント管理

- アカウントの作成・編集・削除
- 複数ユーザーでのアカウント共有
- 共有メンバーの招待・管理

### 3. 収支記録

- 収入の登録・編集・削除
- 支出の登録・編集・削除
- 日付、金額、カテゴリ、メモの入力
- 対応通貨: 日本円のみ

### 4. カテゴリ管理

- ユーザーによるカテゴリのカスタマイズ
- カテゴリの追加・編集・削除
- デフォルトカテゴリの提供

### 5. レポート・グラフ

- 月次の収支サマリー
- カテゴリ別支出の円グラフ
- 月別推移の棒グラフ・折れ線グラフ

### 6. データエクスポート

- JSON 形式でのエクスポート

## データモデル

### User

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | ユーザー ID (Firebase Auth UID) |
| email | string | メールアドレス |
| displayName | string | 表示名 |
| photoURL | string? | プロフィール画像 URL |
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

### アカウント

- `GET /accounts` - アカウント一覧取得
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

- `GET /accounts/:id/transactions` - 取引一覧取得 (フィルタ・ページング対応)
- `POST /accounts/:id/transactions` - 取引作成
- `PATCH /accounts/:id/transactions/:transactionId` - 取引更新
- `DELETE /accounts/:id/transactions/:transactionId` - 取引削除

### レポート

- `GET /accounts/:id/reports/summary` - 収支サマリー取得
- `GET /accounts/:id/reports/by-category` - カテゴリ別集計取得
- `GET /accounts/:id/reports/monthly-trend` - 月別推移取得

### エクスポート

- `GET /accounts/:id/export/json` - JSON エクスポート

## 画面一覧

1. ログイン画面
2. ダッシュボード (アカウント選択・サマリー表示)
3. 収支一覧画面
4. 収支登録・編集画面
5. カテゴリ管理画面
6. レポート画面
7. アカウント設定画面 (メンバー管理含む)
8. ユーザー設定画面

## 非機能要件

- レスポンシブデザイン (モバイル対応)
- 日本語 UI
- セキュアな認証・認可
