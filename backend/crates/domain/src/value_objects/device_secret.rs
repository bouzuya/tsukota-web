use std::fmt;
use std::str::FromStr;

/// デバイスシークレットの最小バイト数
const MIN_LENGTH: usize = 32;

/// デバイスシークレットの Value Object
/// 32 バイト以上の文字列であることを保証する
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceSecret(String);

/// DeviceSecret のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("DeviceSecret must be at least {MIN_LENGTH} bytes")]
pub struct ParseDeviceSecretError;

impl FromStr for DeviceSecret {
    type Err = ParseDeviceSecretError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < MIN_LENGTH {
            return Err(ParseDeviceSecretError);
        }
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for DeviceSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_secret_parse() -> anyhow::Result<()> {
        // 32 バイト以上の文字列
        let secret: DeviceSecret = "01234567890123456789012345678901".parse()?;
        assert_eq!(secret.to_string(), "01234567890123456789012345678901");
        Ok(())
    }

    #[test]
    fn test_device_secret_empty_fails() -> anyhow::Result<()> {
        let result = "".parse::<DeviceSecret>();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_device_secret_too_short_fails() -> anyhow::Result<()> {
        // 31 バイト（最小値未満）
        let result = "0123456789012345678901234567890".parse::<DeviceSecret>();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_device_secret_minimum_length() -> anyhow::Result<()> {
        // ちょうど 32 バイト
        let secret: DeviceSecret = "01234567890123456789012345678901".parse()?;
        assert_eq!(secret.to_string().len(), 32);
        Ok(())
    }

    #[test]
    fn test_device_secret_roundtrip() -> anyhow::Result<()> {
        let secret1: DeviceSecret = "test-secret-value-for-roundtrip!".parse()?;
        let secret_str = secret1.to_string();
        let secret2: DeviceSecret = secret_str.parse()?;
        assert_eq!(secret1, secret2);
        Ok(())
    }

    #[test]
    fn test_device_secret_equality() -> anyhow::Result<()> {
        let secret1: DeviceSecret = "same-secret-value-32-bytes-long!".parse()?;
        let secret2: DeviceSecret = "same-secret-value-32-bytes-long!".parse()?;
        let secret3: DeviceSecret = "different-secret-32-bytes-long!!".parse()?;

        assert_eq!(secret1, secret2);
        assert_ne!(secret1, secret3);
        Ok(())
    }

    #[test]
    fn test_device_secret_with_special_chars() -> anyhow::Result<()> {
        let secret: DeviceSecret = "abc!@#$%^&*()_+-=[]{}|;':\",./<>?".parse()?;
        assert_eq!(secret.to_string(), "abc!@#$%^&*()_+-=[]{}|;':\",./<>?");
        Ok(())
    }
}
