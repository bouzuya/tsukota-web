/// アカウント ID の Value Object
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccountId(uuid::Uuid);

/// AccountId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid AccountId format")]
pub struct ParseAccountIdError;

impl AccountId {
    /// 新しい AccountId を生成する
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// 文字列から AccountId をパースする
    pub fn parse(s: &str) -> Result<Self, ParseAccountIdError> {
        uuid::Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| ParseAccountIdError)
    }

    /// 文字列表現を取得する
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<AccountId> for String {
    fn from(value: AccountId) -> Self {
        value.0.to_string()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_creation() -> anyhow::Result<()> {
        let id = AccountId::new();
        // 文字列表現が有効な形式であることを確認
        let id_str = id.to_string();
        assert!(AccountId::parse(&id_str).is_ok());
        Ok(())
    }

    #[test]
    fn test_account_id_parse() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = AccountId::parse(id_str)?;
        assert_eq!(id.to_string(), id_str);
        Ok(())
    }

    #[test]
    fn test_account_id_roundtrip() -> anyhow::Result<()> {
        let id1 = AccountId::new();
        let id_str = id1.to_string();
        let id2 = AccountId::parse(&id_str)?;
        assert_eq!(id1, id2);
        Ok(())
    }

    #[test]
    fn test_account_id_into_string() -> anyhow::Result<()> {
        let id_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = AccountId::parse(id_str)?;
        let s: String = id.into();
        assert_eq!(s, id_str);
        Ok(())
    }

    #[test]
    fn test_account_id_equality() -> anyhow::Result<()> {
        let id1 = AccountId::parse("550e8400-e29b-41d4-a716-446655440000")?;
        let id2 = AccountId::parse("550e8400-e29b-41d4-a716-446655440000")?;
        let id3 = AccountId::parse("6ba7b810-9dad-11d1-80b4-00c04fd430c8")?;

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        Ok(())
    }
}
