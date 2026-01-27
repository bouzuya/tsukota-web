use std::fmt;

use chrono::SecondsFormat;

/// RFC3339 形式のタイムスタンプを表す Value Object
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DateTime(String);

impl DateTime {
    /// 現在の UTC 時刻を RFC3339 形式（ミリ秒精度）で取得する
    pub fn now() -> Self {
        Self(chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_time_now() -> anyhow::Result<()> {
        let timestamp = DateTime::now();
        let s = timestamp.to_string();
        // RFC3339 形式であることを確認
        assert!(s.contains("T"));
        assert!(s.ends_with("Z"));
        // ミリ秒精度であることを確認（小数点以下3桁）
        let dot_pos = s.find('.').expect("should contain decimal point");
        let z_pos = s.find('Z').expect("should end with Z");
        assert_eq!(z_pos - dot_pos - 1, 3, "should have 3 decimal places");
        Ok(())
    }
}
