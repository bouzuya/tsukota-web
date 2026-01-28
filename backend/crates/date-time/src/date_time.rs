use std::fmt;
use std::str::FromStr;

use chrono::SecondsFormat;
use chrono::SubsecRound;
use chrono::Utc;

/// RFC3339 形式のタイムスタンプを表す Value Object
///
/// 内部的には常に UTC でミリ秒精度で保持する
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DateTime(chrono::DateTime<Utc>);

/// DateTime のパースエラー
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseDateTimeError;

impl fmt::Display for ParseDateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid datetime format")
    }
}

impl std::error::Error for ParseDateTimeError {}

impl DateTime {
    /// 現在の UTC 時刻をミリ秒精度で取得する
    pub fn now() -> Self {
        Self(Utc::now().trunc_subsecs(3))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339_opts(SecondsFormat::Millis, true))
    }
}

impl FromStr for DateTime {
    type Err = ParseDateTimeError;

    /// RFC3339 形式の文字列をパースする
    ///
    /// タイムゾーンオフセット付き（例: "+09:00"）の文字列も受け付け、
    /// UTC に変換して保持する。ミリ秒より細かい精度の場合はエラーを返す。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt = chrono::DateTime::parse_from_rfc3339(s).map_err(|_| ParseDateTimeError)?;
        let utc = dt.with_timezone(&Utc);
        let truncated = utc.trunc_subsecs(3);
        // ミリ秒より細かい精度がある場合はエラー
        if utc != truncated {
            return Err(ParseDateTimeError);
        }
        Ok(Self(truncated))
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

    #[test]
    fn test_parse_utc() -> anyhow::Result<()> {
        let dt: DateTime = "2024-01-15T10:30:45.123Z".parse()?;
        assert_eq!(dt.to_string(), "2024-01-15T10:30:45.123Z");
        Ok(())
    }

    #[test]
    fn test_parse_with_offset() -> anyhow::Result<()> {
        // +09:00 のタイムゾーンは UTC に変換される
        let dt: DateTime = "2024-01-15T19:30:45.123+09:00".parse()?;
        assert_eq!(dt.to_string(), "2024-01-15T10:30:45.123Z");
        Ok(())
    }

    #[test]
    fn test_parse_with_negative_offset() -> anyhow::Result<()> {
        // -05:00 のタイムゾーンは UTC に変換される
        let dt: DateTime = "2024-01-15T05:30:45.123-05:00".parse()?;
        assert_eq!(dt.to_string(), "2024-01-15T10:30:45.123Z");
        Ok(())
    }

    #[test]
    fn test_parse_without_millis() -> anyhow::Result<()> {
        // ミリ秒なしの入力もパース可能（出力はミリ秒精度）
        let dt: DateTime = "2024-01-15T10:30:45Z".parse()?;
        assert_eq!(dt.to_string(), "2024-01-15T10:30:45.000Z");
        Ok(())
    }

    #[test]
    fn test_parse_invalid() {
        let result: Result<DateTime, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_microseconds_fails() {
        // マイクロ秒精度（ミリ秒より細かい）はエラー
        let result: Result<DateTime, _> = "2024-01-15T10:30:45.123456Z".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_trailing_zeros_ok() -> anyhow::Result<()> {
        // ミリ秒以下がゼロならOK（実質ミリ秒精度）
        let dt: DateTime = "2024-01-15T10:30:45.123000Z".parse()?;
        assert_eq!(dt.to_string(), "2024-01-15T10:30:45.123Z");
        Ok(())
    }

    #[test]
    fn test_copy() {
        let dt1 = DateTime::now();
        let dt2 = dt1; // Copy
        assert_eq!(dt1, dt2);
    }

    #[test]
    fn test_ord() -> anyhow::Result<()> {
        let dt1: DateTime = "2024-01-15T10:30:45.123Z".parse()?;
        let dt2: DateTime = "2024-01-15T10:30:45.124Z".parse()?;
        assert!(dt1 < dt2);
        Ok(())
    }
}
