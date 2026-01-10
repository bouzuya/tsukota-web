use std::fmt;
use std::str::FromStr;

/// アカウント ID の Value Object
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccountId(uuid::Uuid);

/// AccountId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid AccountId format")]
pub struct ParseAccountIdError;

impl AccountId {
    /// 新しい AccountId を生成する
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl FromStr for AccountId {
    type Err = ParseAccountIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| ParseAccountIdError)
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_creation() -> anyhow::Result<()> {
        let id = AccountId::generate();
        // 文字列表現が有効な形式であることを確認
        let id_str = id.to_string();
        assert!(id_str.parse::<AccountId>().is_ok());
        Ok(())
    }

    #[test]
    fn test_account_id_parse() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: AccountId = id_str.parse()?;
        assert_eq!(id.to_string(), id_str);
        Ok(())
    }

    #[test]
    fn test_account_id_roundtrip() -> anyhow::Result<()> {
        let id1 = AccountId::generate();
        let id_str = id1.to_string();
        let id2: AccountId = id_str.parse()?;
        assert_eq!(id1, id2);
        Ok(())
    }

    #[test]
    fn test_account_id_into_string() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: AccountId = id_str.parse()?;
        let s = id.to_string();
        assert_eq!(s, id_str);
        Ok(())
    }

    #[test]
    fn test_account_id_equality() -> anyhow::Result<()> {
        let id1: AccountId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id2: AccountId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id3: AccountId = "6ba7b810-9dad-11d1-80b4-00c04fd430c8".parse()?;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        Ok(())
    }
}
