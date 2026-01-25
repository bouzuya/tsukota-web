use std::fmt;
use std::str::FromStr;

/// デバイスシークレットの Value Object
/// 空でない任意の文字列であることを保証する
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DeviceSecret(String);

/// DeviceSecret のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("DeviceSecret cannot be empty")]
pub struct ParseDeviceSecretError;

impl FromStr for DeviceSecret {
    type Err = ParseDeviceSecretError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
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
        let secret: DeviceSecret = "my-secret-value".parse()?;
        assert_eq!(secret.to_string(), "my-secret-value");
        Ok(())
    }

    #[test]
    fn test_device_secret_empty_fails() -> anyhow::Result<()> {
        let result = "".parse::<DeviceSecret>();
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_device_secret_roundtrip() -> anyhow::Result<()> {
        let secret1: DeviceSecret = "test-secret-123".parse()?;
        let secret_str = secret1.to_string();
        let secret2: DeviceSecret = secret_str.parse()?;
        assert_eq!(secret1, secret2);
        Ok(())
    }

    #[test]
    fn test_device_secret_equality() -> anyhow::Result<()> {
        let secret1: DeviceSecret = "same-secret".parse()?;
        let secret2: DeviceSecret = "same-secret".parse()?;
        let secret3: DeviceSecret = "different-secret".parse()?;

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
