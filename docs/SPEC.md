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
| 認証 | デバイス認証 + JWT |
| 状態管理 | jotai |
| CSS | Tailwind CSS |
| デプロイ | Cloud Run / Docker (ローカル) |

詳細なアーキテクチャ設計については [ARCHITECTURE.md](./ARCHITECTURE.md) を参照してください。

## 機能一覧

### 1. ユーザー認証

- デバイス ID とデバイスシークレットによるサインイン
- サインアウト機能

### 2. アカウント管理

- アカウントの作成・編集・削除
- 複数ユーザーでのアカウント共有
- 複数オーナーによる共同管理
- 共有オーナーの招待・管理 (※初期バージョンでは保留、DB直接操作で対応)
- すべてのオーナーが同等の権限を持つ: アカウント削除、オーナー追加・削除、収支・区分管理

### 3. 収支記録

- 取引の登録・編集・削除
- 日付、金額、区分、コメントの入力
- 対応通貨: 日本円のみ
- すべてのオーナーが編集・削除可能

### 4. 区分管理

- オーナーによる区分のカスタマイズ
- 区分の追加・編集・削除 (論理削除)
- 削除された区分: 一覧に非表示、新規取引で選択不可、既存取引の値は維持
- デフォルト区分なし (オーナーが自身で作成)

### 5. データエクスポート

- JSON 形式でのエクスポート (月別)

## データモデル

以下のデータモデルは API レスポンスおよび読み取りモデル (Projection) を表します。
アカウント、区分、取引の実体はイベントソーシングにより管理されます（詳細は [ARCHITECTURE.md](./ARCHITECTURE.md) 参照）。

### User

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | ユーザー ID (UUID) |

### Account (アカウント)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | アカウント ID (UUID) |
| name | string | アカウント名 |
| owner_ids | string[] | オーナーのユーザー ID 一覧 |
| created_at | string | 作成日時 (ISO 8601) |
| updated_at | string | 更新日時 (ISO 8601) |

### Category

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | 区分 ID (UUID) |
| account_id | string | 所属するアカウント ID |
| name | string | 区分名 |
| created_at | string | 作成日時 (ISO 8601) |
| deleted_at | string? | 削除日時 (論理削除、ISO 8601) |

### Transaction (収支記録)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| id | string | 取引 ID (UUID) |
| account_id | string | 所属するアカウント ID |
| amount | string | 金額 (文字列形式) |
| category_id | string | 区分 ID |
| date | string | 取引日 (YYYY-MM-DD) |
| comment | string | コメント |
| created_at | string | 作成日時 (ISO 8601) |
| updated_at | string | 更新日時 (ISO 8601) |

### PaginatedList (ページネーション)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| items | T[] | データの配列 |
| next_cursor | string? | 次ページのカーソル (null の場合は最終ページ) |

## API エンドポイント

API は更新系と参照系で異なるパス設計を採用しています。

- **更新系 (Commands)**: `POST /commands/{use_case_name}` - すべてのパラメーターをリクエストボディに含める
- **参照系 (Queries)**: リソースベースのパス - `GET /accounts/{id}/...`

### コマンドレスポンス形式

作成系コマンドは作成されたリソースの ID を返します (201 Created)。更新・削除系コマンドは空のレスポンス (204 No Content) を返します。

| コマンド | ステータス | レスポンス |
|----------|-----------|-----------|
| `create_account` | 201 | `{ "account_id": string }` |
| `add_category` | 201 | `{ "category_id": string }` |
| `add_transaction` | 201 | `{ "transaction_id": string }` |
| その他 (update/delete) | 204 | 空 |

### 認証

- `POST /commands/create_session_token` - セッショントークン作成
  - body: `{ "device_id": string, "device_secret": string }`
  - response: `{ "session_token": string }`
  - 備考: device_id は UUID v4 形式、device_secret は 32 バイト以上の文字列

### ユーザー

- `GET /me` - 現在のユーザー情報取得
  - response: `{ "user_id": string }`

### アカウント (参照系)

- `GET /accounts` - アカウント一覧取得 (オーナーのアカウント)
  - response: `PaginatedList<Account>`
- `GET /accounts/{account_id}` - アカウント詳細取得
  - response: `Account`

### アカウント (更新系)

- `POST /commands/create_account` - アカウント作成
  - body: `{ "name": string }`
  - response: `{ "account_id": string }`
- `POST /commands/update_account` - アカウント更新
  - body: `{ "account_id": string, "name": string }`
- `POST /commands/delete_account` - アカウント削除
  - body: `{ "account_id": string }`
- `POST /commands/add_owner` - オーナー追加
  - body: `{ "account_id": string, "user_id": string }`
- `POST /commands/remove_owner` - オーナー削除
  - body: `{ "account_id": string, "user_id": string }`

### 区分 (参照系)

- `GET /accounts/{account_id}/categories` - 区分一覧取得
  - response: `PaginatedList<Category>`

### 区分 (更新系)

- `POST /commands/add_category` - 区分作成
  - body: `{ "account_id": string, "name": string }`
  - response: `{ "category_id": string }`
- `POST /commands/update_category` - 区分更新
  - body: `{ "account_id": string, "category_id": string, "name": string }`
- `POST /commands/delete_category` - 区分削除
  - body: `{ "account_id": string, "category_id": string }`

### 収支記録 (参照系)

- `GET /accounts/{account_id}/transactions` - 取引一覧取得
  - query: `?after=<cursor>` (オプション)
  - ページサイズ: 20件、日付降順
  - response: `PaginatedList<Transaction>`

### 収支記録 (更新系)

- `POST /commands/add_transaction` - 取引作成
  - body: `{ "account_id": string, "amount": string, "category_id": string, "comment": string, "date": string }`
  - response: `{ "transaction_id": string }`
- `POST /commands/update_transaction` - 取引更新
  - body: `{ "account_id": string, "transaction_id": string, "amount": string, "category_id": string, "comment": string, "date": string }`
- `POST /commands/delete_transaction` - 取引削除
  - body: `{ "account_id": string, "transaction_id": string }`

### エクスポート (参照系)

- `GET /accounts/{account_id}/export/json` - JSON エクスポート (月別)
  - query: `?year=YYYY&month=MM` (必須)
  - response: `PaginatedList<Transaction>`

## 画面一覧

1. サインイン画面
2. ダッシュボード (アカウント選択)
3. 収支一覧画面
4. 収支登録・編集画面
5. 区分管理画面
6. アカウント設定画面 (オーナー管理含む)
7. ユーザー設定画面

## 非機能要件

- レスポンシブデザイン (モバイル対応)
- 日本語 UI
- セキュアな認証・認可
- API エラーレスポンス: RFC 9457 (Problem Details for HTTP APIs) 準拠
