# Plan: date-time crate の追加

## 概要

`backend/crates` に `date-time` crate を新規追加し、chrono への依存を集約する。

## 現状分析

- **chrono 依存**: `domain` crate のみ
- **使用箇所**: 3箇所、すべて同一パターン `chrono::Utc::now().to_rfc3339()`
  - `domain/src/aggregates/account/aggregate.rs:603`
  - `domain/src/aggregates/user/aggregate.rs:91`
  - `domain/src/aggregates/device/aggregate.rs:128`

## 実装計画

### 1. date-time crate の作成

**ディレクトリ構造:**
```
backend/crates/date-time/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Cargo.toml:**
```toml
[package]
name = "date-time"
authors.workspace = true
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
chrono.workspace = true
```

**src/lib.rs:**
```rust
use std::fmt;

/// RFC3339 形式のタイムスタンプを表す Value Object
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime(String);

impl DateTime {
    /// 現在の UTC 時刻を RFC3339 形式で取得する
    pub fn now() -> Self {
        Self(chrono::Utc::now().to_rfc3339())
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<DateTime> for String {
    fn from(value: DateTime) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_time_now() -> anyhow::Result<()> {
        let timestamp = DateTime::now();
        let s = String::from(timestamp);
        // RFC3339 形式であることを簡易的に確認
        assert!(s.contains("T"));
        assert!(s.contains("+") || s.ends_with("Z"));
        Ok(())
    }
}
```

### 2. ワークスペース設定の更新

**backend/Cargo.toml** に追加:
```toml
date-time = { path = "crates/date-time" }
```

### 3. domain crate の更新

**backend/crates/domain/Cargo.toml**:
- `chrono.workspace = true` を削除
- `date-time.workspace = true` を追加

**aggregate ファイル (3箇所)**:
```rust
// Before:
at: chrono::Utc::now().to_rfc3339(),

// After:
at: date_time::DateTime::now().into(),
```

## 変更対象ファイル

1. **新規作成**
   - `backend/crates/date-time/Cargo.toml`
   - `backend/crates/date-time/src/lib.rs`

2. **修正**
   - `backend/Cargo.toml` - workspace dependency 追加
   - `backend/crates/domain/Cargo.toml` - chrono → date-time
   - `backend/crates/domain/src/aggregates/account/aggregate.rs:603`
   - `backend/crates/domain/src/aggregates/user/aggregate.rs:91`
   - `backend/crates/domain/src/aggregates/device/aggregate.rs:128`

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
