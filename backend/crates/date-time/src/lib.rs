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
