use std::fmt;
use std::str::FromStr;

/// Google アカウント識別子 (OIDC の `sub`) を表す Value Object
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GoogleUserId(String);

/// GoogleUserId のパースエラー
#[derive(Debug, thiserror::Error)]
#[error("Invalid GoogleUserId format")]
pub struct GoogleUserIdError;

impl FromStr for GoogleUserId {
    type Err = GoogleUserIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(GoogleUserIdError);
        }
        if trimmed.chars().any(|c| c.is_control()) {
            return Err(GoogleUserIdError);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl fmt::Display for GoogleUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() -> anyhow::Result<()> {
        let id: GoogleUserId = "1234567890".parse()?;
        assert_eq!(id.to_string(), "1234567890");
        Ok(())
    }

    #[test]
    fn test_parse_empty_rejected() -> anyhow::Result<()> {
        match "".parse::<GoogleUserId>() {
            Err(GoogleUserIdError) => Ok(()),
            Ok(_) => anyhow::bail!("Expected error for empty input"),
        }
    }

    #[test]
    fn test_parse_whitespace_only_rejected() -> anyhow::Result<()> {
        match "   ".parse::<GoogleUserId>() {
            Err(GoogleUserIdError) => Ok(()),
            Ok(_) => anyhow::bail!("Expected error for whitespace input"),
        }
    }

    #[test]
    fn test_parse_control_character_rejected() -> anyhow::Result<()> {
        match "abc\ndef".parse::<GoogleUserId>() {
            Err(GoogleUserIdError) => Ok(()),
            Ok(_) => anyhow::bail!("Expected error for control character"),
        }
    }

    #[test]
    fn test_roundtrip() -> anyhow::Result<()> {
        let id1: GoogleUserId = "test-sub-123".parse()?;
        let id2: GoogleUserId = id1.to_string().parse()?;
        assert_eq!(id1, id2);
        Ok(())
    }
}
