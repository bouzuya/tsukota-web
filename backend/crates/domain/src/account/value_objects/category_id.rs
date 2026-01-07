use std::fmt;
use std::str::FromStr;

/// カテゴリ ID の Value Object
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CategoryId(uuid::Uuid);

/// CategoryId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid CategoryId format")]
pub struct ParseCategoryIdError;

impl CategoryId {
    /// 新しい CategoryId を生成する
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl FromStr for CategoryId {
    type Err = ParseCategoryIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| ParseCategoryIdError)
    }
}

impl fmt::Display for CategoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<CategoryId> for String {
    fn from(value: CategoryId) -> Self {
        value.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_id_creation() -> anyhow::Result<()> {
        let id = CategoryId::new();
        // 文字列表現が有効な形式であることを確認
        let id_str = id.to_string();
        assert!(id_str.parse::<CategoryId>().is_ok());
        Ok(())
    }

    #[test]
    fn test_category_id_parse() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: CategoryId = id_str.parse()?;
        assert_eq!(id.to_string(), id_str);
        Ok(())
    }

    #[test]
    fn test_category_id_roundtrip() -> anyhow::Result<()> {
        let id1 = CategoryId::new();
        let id_str = id1.to_string();
        let id2: CategoryId = id_str.parse()?;
        assert_eq!(id1, id2);
        Ok(())
    }

    #[test]
    fn test_category_id_into_string() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: CategoryId = id_str.parse()?;
        let s: String = id.into();
        assert_eq!(s, id_str);
        Ok(())
    }

    #[test]
    fn test_category_id_equality() -> anyhow::Result<()> {
        let id1: CategoryId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id2: CategoryId = "550e8400-e29b-41d4-a716-446655440000".parse()?;
        let id3: CategoryId = "6ba7b810-9dad-11d1-80b4-00c04fd430c8".parse()?;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        Ok(())
    }
}
