# tsukota-web コーディングガイドライン

> このドキュメントは、tsukota-web プロジェクトのコーディングスタイル、ルール、ベストプラクティスを定義します。

## プロジェクト構造

### Cargo Workspace

```
backend/
├── Cargo.toml              # Workspace 設定
└── crates/
    ├── domain/             # ドメイン層
    ├── application/        # アプリケーション層
    ├── infra/              # インフラ層
    └── api/                # プレゼンテーション層
```

- **命名規則**: `crates/{crate_name}` の形式で配置
- **依存関係管理**: `[workspace.dependencies]` で共通の依存関係を定義
- **バージョン管理**: workspace レベルで統一 (`version.workspace = true`)

### Cargo.toml の設定

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
version = "0.0.0"
edition = "2024"
authors = ["bouzuya"]
license = "MIT OR Apache-2.0"
```

### モジュール構成

#### モジュールファイルの配置

**ルール**: `mod.rs` を使用せず、名前付きモジュールファイルを使用する

```
src/
├── lib.rs
├── account.rs          # account モジュールの宣言
└── account/
    ├── aggregate.rs
    ├── commands.rs
    ├── events.rs
    ├── value_objects.rs    # value_objects サブモジュールの宣言
    └── value_objects/
        ├── account_id.rs
        ├── category_id.rs
        ├── transaction_id.rs
        └── user_id.rs
```

**account.rs の例**:

```rust
mod aggregate;
mod commands;
mod events;
mod value_objects;

pub use aggregate::*;
pub use commands::*;
pub use events::*;
pub use value_objects::*;
```

**account/value_objects.rs の例**:

```rust
mod account_id;
mod category_id;
mod transaction_id;
mod user_id;

pub use account_id::AccountId;
pub use account_id::ParseAccountIdError;
pub use category_id::CategoryId;
pub use category_id::ParseCategoryIdError;
pub use transaction_id::ParseTransactionIdError;
pub use transaction_id::TransactionId;
pub use user_id::ParseUserIdError;
pub use user_id::UserId;
```

**ルール**:

- `mod.rs` は使用しない（`account/mod.rs` ではなく `account.rs`）
- Rust は自動的に `account.rs` の隣の `account/` ディレクトリからサブモジュールを探す
- `#[path = "..."]` 属性は不要（Rust の規約に従う）
- 型とそのエラー型は同じモジュールに配置
- モジュールファイルでは個別に `pub use` を記述（グループ化しない）
- alphabetical order で記述

#### lib.rs の構成ルール

**ルール**: `lib.rs` には `mod` 宣言と `pub use` のみを記述し、実装コードは配置しない

```rust
// ✅ 推奨: lib.rs
mod date_time;

pub use self::date_time::DateTime;
pub use self::date_time::ParseDateTimeError;
```

```rust
// ❌ 避ける: lib.rs に実装コードを直接配置
pub struct DateTime(chrono::DateTime<Utc>);

impl DateTime {
    pub fn now() -> Self {
        // ...
    }
}
```

**理由**:
- モジュール構造の一貫性を保つ
- 各モジュールの責務を明確にする
- テストコードを含む実装を独立したファイルに分離できる

## Rust コーディングスタイル

### 0. インポート形式

**ルール**: インポートは `imports_granularity = "Item"` に従い、個別に記述する

```rust
// ❌ 避ける
use std::collections::{BTreeMap, BTreeSet};
use super::value_objects::{AccountId, CategoryId};

// ✅ 推奨
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use super::value_objects::AccountId;
use super::value_objects::CategoryId;
```

**設定**: `rustfmt.toml` に以下を記述

```toml
unstable_features = true
imports_granularity = "Item"
```

**フォーマット**: `cargo +nightly fmt` で自動整形

### 1. derive マクロの順序

**ルール**: derive マクロは alphabetical order で記述する

```rust
// ✅ 推奨
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccountId(uuid::Uuid);

// ❌ 避ける
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccountId(uuid::Uuid);
```

**理由**: 順序の一貫性を保ち、視認性を向上させる

### 2. コレクション型の選択

**ルール**: `HashSet` / `HashMap` ではなく `BTreeSet` / `BTreeMap` を使用する

**理由**: 実行時の効率よりデバッグ時の並び順を重視

```rust
// ❌ 避ける
use std::collections::{HashMap, HashSet};

// ✅ 推奨
use std::collections::{BTreeMap, BTreeSet};
```

### 3. パターンマッチング

**ルール**: `if let` ではなく `match` を使用し、到達を想定していない箇所は `unreachable!()` で明示

```rust
// ❌ 避ける
if let Account::Active { name: current_name, .. } = self {
    *current_name = name.clone();
}

// ✅ 推奨
match self {
    Account::Active { name: current_name, .. } => {
        *current_name = name.clone();
    }
    Account::Empty => unreachable!("AccountUpdated event applied to Empty account"),
}
```

**理由**: すべてのケースを明示的に扱い、到達不可能なコードを文書化する

### 4. テストコードのエラーハンドリング

**ルール**: テストは `anyhow::Result<()>` を返し、`unwrap()`, `expect()`, `panic!()` の使用を避ける

```rust
// ❌ 避ける
#[test]
fn test_example() {
    let result = do_something().unwrap();
    assert_eq!(result, expected);
}

// ✅ 推奨
#[test]
fn test_example() -> anyhow::Result<()> {
    let result = do_something()?;
    assert_eq!(result, expected);
    Ok(())
}
```

**match でのエラーハンドリング**:

```rust
match result {
    Ok(value) => {
        assert_eq!(value, expected);
        Ok(())
    }
    Err(e) => anyhow::bail!("Expected success, got error: {:?}", e),
}
```

### 5. 状態の型安全性

**ルール**: `Option<T>` による null チェックではなく、enum で状態を表現する

```rust
// ❌ 避ける
pub struct Account {
    pub id: Option<AccountId>,
    pub name: Option<String>,
    // ...
}

// ✅ 推奨
pub enum Account {
    Empty,
    Active {
        id: AccountId,
        name: String,
        // ...
    },
}
```

**理由**: 型システムで状態を保証し、null チェックを不要にする

## イベントソーシング

### tsukota との互換性

**ルール**: イベント定義は [tsukota](https://github.com/bouzuya/tsukota) の `account-events.ts` と同一の形式を保つ

- イベント名: `AccountCreated`, `OwnerAdded`, `CategoryAdded`, `TransactionAdded` 等
- 共通プロパティ: `accountId`, `at`, `id`, `protocolVersion`
- TransactionProps: `amount` (string), `categoryId`, `comment`, `date`

### 集約の設計

**ルール**: 集約の単位は **Account のみ**

- Category と Transaction は Account の一部として扱う
- User はイベントソーシングを使用しない

### イベント適用

```rust
pub fn apply_event(&mut self, event: &AccountEvent) {
    match event {
        AccountEvent::AccountCreated { name, owners, .. } => {
            // 状態遷移のロジック
        }
        AccountEvent::SomeEvent { .. } => match self {
            Account::Active { field, .. } => {
                // フィールドの更新
            }
            Account::Empty => unreachable!("SomeEvent applied to Empty account"),
        },
    }
}
```

## 命名規則

### イベント名

- **過去形**: `AccountCreated`, `OwnerAdded`, `CategoryDeleted`
- **camelCase**: JSON シリアライズ時は `accountCreated`, `ownerAdded`

### コマンド名

- **動詞 + 名詞**: `CreateAccount`, `AddOwner`, `DeleteCategory`
- **Update vs Rename**: 汎用的な更新は `Update`, 名前のみは `Rename` (ただし tsukota に合わせて `Update` を使用)

### Value Objects

**ルール**: ID 型は型エイリアスではなく、Value Object として定義する

すべての ID 型（`AccountId`, `UserId`, `CategoryId`, `TransactionId`）は newtype パターンで実装し、以下の trait を実装する：

- `FromStr`: 文字列からのパース
- `Display`: 文字列への変換
- `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`: 標準的な派生

**実装例**:

```rust
use std::fmt;
use std::str::FromStr;

/// アカウント ID の Value Object
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccountId(uuid::Uuid);

/// AccountId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid AccountId format")]
pub struct AccountIdError;

impl AccountId {
    /// 新しい AccountId を生成する
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl FromStr for AccountId {
    type Err = AccountIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| AccountIdError)
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**使用方法**:

```rust
// パース
let id: AccountId = "550e8400-e29b-41d4-a716-446655440000".parse()?;

// 文字列への変換
let id_str = id.to_string();

// 新規生成
let new_id = AccountId::new();
```

**注意**:

- `From<AccountId> for String` は実装しない（`Display` trait で十分）
- カスタムメソッド `parse()` や `to_string()` は定義せず、標準 trait を使用
- イベントとの境界では `to_string()` / `.parse()` で変換

## レイヤー間の依存関係

```
api → application → domain
        ↓
      infra → domain
```

**ルール**:
- `domain` は他のどのレイヤーにも依存しない（依存性逆転の原則）
- `infra` は `domain` の trait を実装
- 上位レイヤーは下位レイヤーに依存可能

## エラーハンドリング

### ドメインエラー

```rust
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("Account already exists")]
    AccountAlreadyExists,

    #[error("Account not found")]
    AccountNotFound,
    // ...
}
```

**ルール**: `thiserror` を使用し、明確なエラーメッセージを定義

### Result 型の使用

```rust
pub fn handle_command(
    &self,
    command: AccountCommand,
) -> Result<Vec<AccountEvent>, AccountError> {
    // ...
}
```

## ドキュメントコメント

### 日本語コメント

プロジェクト内のコメントは日本語で記述する

```rust
/// アカウント集約に対するイベント
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountEvent {
    /// アカウントが作成された
    AccountCreated {
        // ...
    },
}
```

## コード品質

### 自動フォーマット

- `rustfmt` を使用
- edition 2024 の設定に従う

### 静的解析

- `clippy` の警告に対応
- `cargo check` で型チェックを実施

### テスト

- ユニットテストは各モジュール内の `#[cfg(test)]` ブロックに配置
- テストは anyhow::Result<()> を返す
- テストケース名は `test_` で始める

## 依存関係

### 主要な依存関係

- **serde**: シリアライゼーション
- **thiserror**: エラー定義
- **anyhow**: テストでのエラーハンドリング
- **chrono**: 日時処理
- **uuid**: ID 生成

### dev-dependencies の分離

テスト専用の依存関係は `[dev-dependencies]` に配置

```toml
[dev-dependencies]
anyhow.workspace = true
```

## まとめ

これらのガイドラインは、コードの一貫性、可読性、保守性を高めるために定義されています。新しいコードを書く際は、既存のコードスタイルに従い、これらのルールを遵守してください。
