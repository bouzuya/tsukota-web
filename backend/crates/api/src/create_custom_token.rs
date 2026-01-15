/// Claims for Firebase custom token
/// <https://firebase.google.com/docs/auth/admin/create-custom-tokens#create_custom_tokens_using_a_third-party_jwt_library> に従って JWT を発行します
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FirebaseCustomTokenClaims {
    /// Issuer - service account email address
    pub iss: String,
    /// Subject - service account email address
    pub sub: String,
    /// Audience - Firebase Identity Toolkit URL
    pub aud: String,
    /// Issued-at time in seconds since UNIX epoch
    pub iat: u64,
    /// Expiration time in seconds since UNIX epoch
    pub exp: u64,
    /// Unique identifier of the user (1-128 characters)
    pub uid: String,
}

/// Error type for custom token creation
#[derive(Debug)]
pub enum CreateCustomTokenError {
    /// UID is empty or exceeds 128 characters
    InvalidUid,
    /// JWT encoding failed
    JwtEncodingError(jsonwebtoken::errors::Error),
    /// System time error
    SystemTimeError,
}

impl std::fmt::Display for CreateCustomTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateCustomTokenError::InvalidUid => {
                write!(f, "UID must be between 1 and 128 characters")
            }
            CreateCustomTokenError::JwtEncodingError(e) => {
                write!(f, "JWT encoding error: {}", e)
            }
            CreateCustomTokenError::SystemTimeError => {
                write!(f, "Failed to get system time")
            }
        }
    }
}

impl std::error::Error for CreateCustomTokenError {}

/// Creates a Firebase custom token for the given user.
///
/// # Arguments
///
/// * `service_account_email` - The service account email address (used as iss and sub)
/// * `private_key_pem` - The RSA private key in PEM format
/// * `uid` - The unique identifier of the user (1-128 characters)
/// * `custom_claims` - Optional custom claims to include in the token
///
/// # Returns
///
/// A JWT string that can be used with Firebase's `signInWithCustomToken`
pub fn create_custom_token(
    service_account_email: &str,
    private_key_pem: &str,
    uid: &str,
) -> Result<String, CreateCustomTokenError> {
    // Validate UID length
    if !(1..=128).contains(&uid.len()) {
        return Err(CreateCustomTokenError::InvalidUid);
    }

    // Get current time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| CreateCustomTokenError::SystemTimeError)?
        .as_secs();

    // Create claims
    let claims = FirebaseCustomTokenClaims {
        iss: service_account_email.to_string(),
        sub: service_account_email.to_string(),
        aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_owned(),
        iat: now,
        exp: now + 3600,
        uid: uid.to_string(),
    };

    // Create header with RS256 algorithm
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

    // Encode the token
    let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(CreateCustomTokenError::JwtEncodingError)?;

    jsonwebtoken::encode(&header, &claims, &encoding_key)
        .map_err(CreateCustomTokenError::JwtEncodingError)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test RSA private key for testing purposes only
    const TEST_PRIVATE_KEY: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA2mKqH0dSgVf1m7KdLwNxFJd5vEuYoLXbR9VlWFy1a2Q8Yk7d
KyHgH5a7L3VzK7gU3VhSvJl8k3xRN+dEu7hN9tGm6fR5N2zFyKV6HfJ3m7VuT5aR
7xDvJZGxMJNF7zFl5rkZP8f4B7VvZD1E8L3z3M6vY7cK9a+FbKx7C7S8D9P2T4jX
qN9d7vR2ZmC3s4FG6F3W1d5V+M8h6N1T7xQ3K2FvS7h3g8X9E2T5vL4R6Y3s8F1W
9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M
7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9
E2T5vL4R6Y3s8F1W9C7B3D2Z8QIDAQABAoIBAC3D5M8F4K9E2T5vL4R6Y3s8F1W9
C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7
xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2
T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7
B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3
K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5v
L4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D
2Z8Q4V6N1MAoGBAP3F4K9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3
F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8
F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6
N1M7xR3K2HAoGBANt5F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7
T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y
3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q
4V6N1M7xR3AoGAV3Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2
Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H
5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4
R6Y3s8F1W9AoGAQ4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4
V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T
3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3
s8F1W9C7B3AoGBAM7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M
7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9
E2T5vL4R6Y3s8F1W9C7B3D2Z8Q4V6N1M7xR3K2H5S7T3F8X9E2T5vL4R6Y3s8F1W
9C7B3D2Z8Q4
-----END RSA PRIVATE KEY-----"#;

    #[test]
    fn test_invalid_uid_empty() {
        let result = create_custom_token("test@example.com", TEST_PRIVATE_KEY, "");
        assert!(matches!(result, Err(CreateCustomTokenError::InvalidUid)));
    }

    #[test]
    fn test_invalid_uid_too_long() {
        let long_uid = "a".repeat(129);
        let result = create_custom_token("test@example.com", TEST_PRIVATE_KEY, &long_uid);
        assert!(matches!(result, Err(CreateCustomTokenError::InvalidUid)));
    }
}
