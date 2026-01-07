use std::fmt;
use std::str::FromStr;

/// ユーザー ID の Value Object
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserId(uuid::Uuid);

/// UserId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid UserId format")]
pub struct ParseUserIdError;

impl UserId {
    /// 新しい UserId を生成する
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl FromStr for UserId {
    type Err = ParseUserIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| ParseUserIdError)
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_creation() -> anyhow::Result<()> {
        let id = UserId::new();
        // 文字列表現が有効な形式であることを確認
        let id_str = id.to_string();
        assert!(id_str.parse::<UserId>().is_ok());
        Ok(())
    }

    #[test]
    fn test_user_id_parse() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: UserId = id_str.parse()?;
        assert_eq!(id.to_string(), id_str);
        Ok(())
    }

    #[test]
    fn test_user_id_roundtrip() -> anyhow::Result<()> {
        let id1 = UserId::new();
        let id_str = id1.to_string();
        let id2: UserId = id_str.parse()?;
        assert_eq!(id1, id2);
        Ok(())
    }

    #[test]
    fn test_user_id_into_string() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: UserId = id_str.parse()?;
        let s = id.to_string();
        assert_eq!(s, id_str);
        Ok(())
    }

    #[test]
    fn test_user_id_equality() -> anyhow::Result<()> {
        let id1: UserId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id2: UserId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id3: UserId = "6ba7b810-9dad-11d1-80b4-00c04fd430c8".parse()?;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        Ok(())
    }
}
