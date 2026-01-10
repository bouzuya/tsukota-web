/// User ID for execution context in use cases
///
/// This type is used in the application layer to represent the authenticated user
/// executing a use case. It is separate from the domain's UserId to maintain
/// architectural boundaries.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UserId(domain::account::UserId);

/// Error when parsing UserId
#[derive(Debug, thiserror::Error)]
#[error("Invalid UserId format")]
pub struct ParseUserIdError;

impl UserId {
    /// Convert to domain UserId
    pub fn to_domain(&self) -> domain::account::UserId {
        self.0
    }
}

impl std::str::FromStr for UserId {
    type Err = ParseUserIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        domain::account::UserId::from_str(s)
            .map(Self)
            .map_err(|_| ParseUserIdError)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
