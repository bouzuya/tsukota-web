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

## Rust コーディングスタイル

### 1. コレクション型の選択

**ルール**: `HashSet` / `HashMap` ではなく `BTreeSet` / `BTreeMap` を使用する

**理由**: 実行時の効率よりデバッグ時の並び順を重視

```rust
// ❌ 避ける
use std::collections::{HashMap, HashSet};

// ✅ 推奨
use std::collections::{BTreeMap, BTreeSet};
```

### 2. パターンマッチング

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

### 3. テストコードのエラーハンドリング

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

### 4. 状態の型安全性

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

### 型エイリアス

```rust
pub type AccountId = String;
pub type UserId = String;
pub type CategoryId = String;
pub type TransactionId = String;
```

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
