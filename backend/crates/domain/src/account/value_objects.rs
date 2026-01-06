/// アカウント ID の Value Object
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AccountId(String);

impl AccountId {
    /// 新しい AccountId を作成する
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 内部の String 値への参照を取得する
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AccountId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<AccountId> for String {
    fn from(value: AccountId) -> Self {
        value.0
    }
}

impl AsRef<str> for AccountId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_id_creation() -> anyhow::Result<()> {
        let id = AccountId::new("account-123");
        assert_eq!(id.as_str(), "account-123");
        Ok(())
    }

    #[test]
    fn test_account_id_from_string() -> anyhow::Result<()> {
        let s = String::from("account-456");
        let id: AccountId = s.into();
        assert_eq!(id.as_str(), "account-456");
        Ok(())
    }

    #[test]
    fn test_account_id_into_string() -> anyhow::Result<()> {
        let id = AccountId::new("account-789");
        let s: String = id.into();
        assert_eq!(s, "account-789");
        Ok(())
    }

    #[test]
    fn test_account_id_equality() -> anyhow::Result<()> {
        let id1 = AccountId::new("account-123");
        let id2 = AccountId::new("account-123");
        let id3 = AccountId::new("account-456");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        Ok(())
    }
}
