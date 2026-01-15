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

/// Error type for signing operations
#[derive(Debug)]
pub enum SignError {
    /// UID is empty or exceeds 128 characters
    InvalidUid,
    /// JWT encoding failed
    JwtEncodingError(jsonwebtoken::errors::Error),
    /// System time error
    SystemTimeError,
}

impl std::fmt::Display for SignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignError::InvalidUid => {
                write!(f, "UID must be between 1 and 128 characters")
            }
            SignError::JwtEncodingError(e) => {
                write!(f, "JWT encoding error: {}", e)
            }
            SignError::SystemTimeError => {
                write!(f, "Failed to get system time")
            }
        }
    }
}

impl std::error::Error for SignError {}

/// Signer for creating Firebase custom tokens
pub struct Signer {
    encoding_key: jsonwebtoken::EncodingKey,
    service_account_email: String,
}

impl Signer {
    /// Creates a new Signer instance.
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - The RSA private key in PEM format
    /// * `service_account_email` - The service account email address (used as iss and sub)
    pub fn new(
        private_key_pem: &str,
        service_account_email: String,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        Ok(Self {
            service_account_email,
            encoding_key,
        })
    }

    /// Returns the current time in seconds since UNIX epoch.
    pub fn now() -> Result<u64, SignError> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|_| SignError::SystemTimeError)
    }

    /// Signs a Firebase custom token for the given user.
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the user (1-128 characters)
    /// * `now` - The current time in seconds since UNIX epoch
    ///
    /// # Returns
    ///
    /// A JWT string that can be used with Firebase's `signInWithCustomToken`
    pub fn sign(&self, uid: &str, now: u64) -> Result<String, SignError> {
        // Validate UID length
        if !(1..=128).contains(&uid.len()) {
            return Err(SignError::InvalidUid);
        }

        // Create claims
        let claims = FirebaseCustomTokenClaims {
            iss: self.service_account_email.clone(),
            sub: self.service_account_email.clone(),
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_owned(),
            iat: now,
            exp: now + 3600,
            uid: uid.to_string(),
        };

        // Create header with RS256 algorithm
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

        // Encode the token
        jsonwebtoken::encode(&header, &claims, &self.encoding_key)
            .map_err(SignError::JwtEncodingError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test RSA public key for testing purposes only (derived from TEST_PRIVATE_KEY_PEM)
    const TEST_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAqYPnaQlwAJJQPUr8KtB0
juDbXbThT+8+vVp8Aqt+iYePeabEK+Pr9cBicsR4fUcqdQmJ259OlIkJb2fpX8Bv
0BJSbDS5JeK7Zt7uu5YLz1LBMiFbE4cEYpYDXjc2NqG7ehiBRl9auqhriFVYgMTL
KJuKgkQuFR2ZQ1RG50fFizZQi1w4YjYc/FhYGovwCIxDOeDvIV1+ew7T6GBBH+EK
okn7FsAtbW3+YLXlpFA3IZ+ur2ixNI/JzXC6zYzus1srsZUezpwnNzW86ChOQxLy
0nvuiOWhXz48a4T/ntYewQ370NcA7FkpYjZXrWFOjvzLoVATEmK+1hJFK3h/+1+0
dwIDAQAB
-----END PUBLIC KEY-----"#;

    // Test RSA private key for testing purposes only
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCpg+dpCXAAklA9
Svwq0HSO4NtdtOFP7z69WnwCq36Jh495psQr4+v1wGJyxHh9Ryp1CYnbn06UiQlv
Z+lfwG/QElJsNLkl4rtm3u67lgvPUsEyIVsThwRilgNeNzY2obt6GIFGX1q6qGuI
VViAxMsom4qCRC4VHZlDVEbnR8WLNlCLXDhiNhz8WFgai/AIjEM54O8hXX57DtPo
YEEf4QqiSfsWwC1tbf5gteWkUDchn66vaLE0j8nNcLrNjO6zWyuxlR7OnCc3Nbzo
KE5DEvLSe+6I5aFfPjxrhP+e1h7BDfvQ1wDsWSliNletYU6O/MuhUBMSYr7WEkUr
eH/7X7R3AgMBAAECggEAAJ9cDDCyrpiFnulVlRAAPZNp0CWdO4k/YRFMqfMt8HIE
/tyz6mKt4nsltrzYocJ5z3A3h2JFtu9B5PIsEzlGWoHsDOWsqwzyxI/xBH3/5m+s
B0kZZ9rzvdZF249hrZWudeZSeIsFGZu2DrbG2mzelMmYGp5cJPZfh7XEL46TP82e
ZKftyWzDwj+EbBse9ycUpYAZKeBf0hnb5OD+mx1REUvVtwgvKvlK53nbCKPkNDEI
wRnY5DJkaq5AbmoQ3IEu8SD/l80frX/FBgd+6b9iFiAjJnDRgXbFzuh9PvCUxLk4
IHq5pKDY+ev93SXlM9rqgViiKrNzCXUTG+ufVsPW0QKBgQDaWG57xkAaY3RnEGsw
4DJ3IMB38Hq8GyjUmWXJgv92rLzN3kDIxyP6Ogzocx9lTgdtQjfWCaStQggUNDja
BOXu0ORs/U2RDyi+bDAOLIgzTJ3g2U/XtOkkLIo1g1bmth8lULtv95/xERj82aJ9
3jRJVudLjr9IFitnof3NJQYCPQKBgQDGv7SI0fpsmakUMDLA/nZgZlcShQ5Wyea2
MPVXld9BwYiOKQD0FSJvH3g5pqIX8XriQgCkhFKz3mBgZQhDlkyyQ1MHe3KZCiVC
EQvQI7NYgsr3y/v2Cnq+hklXH++S1+NL1Exd4Fh8/oipv2+6Q3MHVItWDOk9Dcnl
sopg37wAwwKBgAZxgaEpco7UzISWGXOxygt17fgcIqMWchgEBtrxgLRx2IiCvIqw
RcGGoQbjDtQgf1ucDqXEVxW90xs1h0/3wQlRJMyKlRVoyx0DeE+SxNousqIGB7Mn
5ZFptxJpM9FPIpApV76wIgotJP2hNohcXFKlu+Gg7sgjz1gZkbHSG+FJAoGBAIk5
4Cr/4EqPpiBcTw7WI2HYB5Kv8ACYkwWEtEFvJ6E9QU32ncPpu8bCEb6sgQrLHq+O
JudwvbGXfy/PUm9oHTiQJ4npAG0Ohj8hieiCXdhlJkMFwshU3/8gtQ7E7COVkEjR
IpFGz5IuJKoflcMjww5yn2ogrAINvTMtnUHZ+PMFAoGASgxWWQbk824A3Wu5qdia
Letmc7GGERRH578lkxqtDRdIcSzrdL6hUnqUAW11hs/NWIThdFSYPJeUyIrVfUFj
Z5L7OHrn5SawnnFX38HdiK1leE8uoDEKMhy+1BePdiQhJJAExv0FxJKcA8tHR7KA
XHoL8lz5DpxcSiLilKDCKxo=
-----END PRIVATE KEY-----"#;

    #[test]
    fn test_invalid_uid_empty() -> anyhow::Result<()> {
        let signer = Signer::new(TEST_PRIVATE_KEY_PEM, "test@example.com".to_string())?;
        let result = signer.sign("", 1000);
        assert!(matches!(result, Err(SignError::InvalidUid)));
        Ok(())
    }

    #[test]
    fn test_invalid_uid_too_long() -> anyhow::Result<()> {
        let signer = Signer::new(TEST_PRIVATE_KEY_PEM, "test@example.com".to_string())?;
        let long_uid = "a".repeat(129);
        let result = signer.sign(&long_uid, 1000);
        assert!(matches!(result, Err(SignError::InvalidUid)));
        Ok(())
    }

    #[test]
    fn test_sign_success() -> anyhow::Result<()> {
        let service_account_email = "test@example.iam.gserviceaccount.com";
        let signer = Signer::new(TEST_PRIVATE_KEY_PEM, service_account_email.to_string())?;
        let uid = "user123";
        let now = 1700000000_u64;

        let token = signer.sign(uid, now)?;

        // Decode and verify the token claims
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = false;
        validation.set_required_spec_claims::<&str>(&[]);
        validation.set_audience(&["https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit"]);
        validation.set_issuer(&[service_account_email]);

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY_PEM.as_bytes())?;
        let token_data = jsonwebtoken::decode::<FirebaseCustomTokenClaims>(&token, &decoding_key, &validation)?;

        assert_eq!(token_data.claims.iss, service_account_email);
        assert_eq!(token_data.claims.sub, service_account_email);
        assert_eq!(token_data.claims.aud, "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit");
        assert_eq!(token_data.claims.iat, now);
        assert_eq!(token_data.claims.exp, now + 3600);
        assert_eq!(token_data.claims.uid, uid);

        Ok(())
    }
}
