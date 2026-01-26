use application::SessionTokenCreator;
use application::SessionTokenVerifier;

/// セッショントークンの JWT クレーム
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SessionTokenClaims {
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

/// トークン作成エラー
#[derive(Debug)]
pub enum CreateError {
    /// UID が空または 128 文字を超えている
    InvalidUid,
    /// JWT エンコードに失敗
    JwtEncodingError(jsonwebtoken::errors::Error),
    /// システム時刻エラー
    SystemTimeError,
}

impl std::fmt::Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateError::InvalidUid => {
                write!(f, "UID must be between 1 and 128 characters")
            }
            CreateError::JwtEncodingError(e) => {
                write!(f, "JWT encoding error: {}", e)
            }
            CreateError::SystemTimeError => {
                write!(f, "Failed to get system time")
            }
        }
    }
}

impl std::error::Error for CreateError {}

/// セッショントークン作成器
#[derive(Clone)]
pub struct Creator {
    encoding_key: jsonwebtoken::EncodingKey,
    service_account_email: String,
}

impl Creator {
    /// 新しい Creator インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - RSA 秘密鍵 (PEM 形式)
    /// * `service_account_email` - サービスアカウントのメールアドレス (iss と sub に使用)
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

    /// 現在時刻を UNIX エポックからの秒数で返す
    pub fn now() -> Result<u64, CreateError> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|_| CreateError::SystemTimeError)
    }

    /// 指定されたユーザーのセッショントークンを作成する
    ///
    /// # Arguments
    ///
    /// * `uid` - ユーザー識別子 (1-128 文字)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    ///
    /// # Returns
    ///
    /// JWT 形式のセッショントークン
    pub fn create(&self, uid: &str, now: u64) -> Result<String, CreateError> {
        // UID の長さを検証
        if !(1..=128).contains(&uid.len()) {
            return Err(CreateError::InvalidUid);
        }

        // クレームを作成
        let claims = SessionTokenClaims {
            iss: self.service_account_email.clone(),
            sub: self.service_account_email.clone(),
            aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_owned(),
            iat: now,
            exp: now + 3600,
            uid: uid.to_string(),
        };

        // RS256 アルゴリズムでヘッダーを作成
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

        // トークンをエンコード
        jsonwebtoken::encode(&header, &claims, &self.encoding_key)
            .map_err(CreateError::JwtEncodingError)
    }
}

#[async_trait::async_trait]
impl SessionTokenCreator for Creator {
    async fn create(
        &self,
        uid: &str,
        now: u64,
    ) -> Result<String, application::session_token::CreatorError> {
        self.create(uid, now)
            .map_err(|e| Box::new(e) as application::session_token::CreatorError)
    }
}

/// トークン検証エラー
#[derive(Debug)]
pub enum VerifyError {
    /// JWT デコードに失敗
    JwtDecodeError(jsonwebtoken::errors::Error),
    /// トークンが期限切れ
    TokenExpired,
    /// システム時刻エラー
    SystemTimeError,
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::JwtDecodeError(e) => {
                write!(f, "JWT decode error: {}", e)
            }
            VerifyError::TokenExpired => {
                write!(f, "Token has expired")
            }
            VerifyError::SystemTimeError => {
                write!(f, "Failed to get system time")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// JWT トークンの検証器
#[derive(Clone)]
pub struct Verifier {
    decoding_key: jsonwebtoken::DecodingKey,
    service_account_email: String,
}

impl Verifier {
    /// 新しい Verifier インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - RSA 秘密鍵 (PEM 形式)
    /// * `service_account_email` - サービスアカウントのメールアドレス
    pub fn new(
        private_key_pem: &str,
        service_account_email: String,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        // 秘密鍵から公開鍵コンポーネントを抽出してデコード用キーを作成
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(
            Self::extract_public_key(private_key_pem)?.as_bytes(),
        )?;
        Ok(Self {
            decoding_key,
            service_account_email,
        })
    }

    /// 秘密鍵から公開鍵を抽出する
    fn extract_public_key(private_key_pem: &str) -> Result<String, jsonwebtoken::errors::Error> {
        use rsa::pkcs8::DecodePrivateKey;
        use rsa::pkcs8::EncodePublicKey;

        let private_key = rsa::RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|e| {
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidRsaKey(
                e.to_string(),
            ))
        })?;

        let public_key = rsa::RsaPublicKey::from(&private_key);
        let public_key_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .map_err(|e| {
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidRsaKey(
                    e.to_string(),
                ))
            })?;

        Ok(public_key_pem)
    }

    /// トークンを検証して UID を取得する
    pub fn verify(&self, token: &str) -> Result<String, VerifyError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&["https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit"]);
        validation.set_issuer(&[&self.service_account_email]);
        // 標準クレームの検証を無効化（uid は標準クレームではない）
        validation.set_required_spec_claims::<&str>(&[]);

        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(token, &self.decoding_key, &validation)
                .map_err(VerifyError::JwtDecodeError)?;

        Ok(token_data.claims.uid)
    }
}

impl SessionTokenVerifier for Verifier {
    fn verify(&self, token: &str) -> Result<String, application::session_token::VerifierError> {
        self.verify(token)
            .map_err(|e| Box::new(e) as application::session_token::VerifierError)
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
        let creator = Creator::new(TEST_PRIVATE_KEY_PEM, "test@example.com".to_string())?;
        let result = creator.create("", 1000);
        assert!(matches!(result, Err(CreateError::InvalidUid)));
        Ok(())
    }

    #[test]
    fn test_invalid_uid_too_long() -> anyhow::Result<()> {
        let creator = Creator::new(TEST_PRIVATE_KEY_PEM, "test@example.com".to_string())?;
        let long_uid = "a".repeat(129);
        let result = creator.create(&long_uid, 1000);
        assert!(matches!(result, Err(CreateError::InvalidUid)));
        Ok(())
    }

    #[test]
    fn test_create_success() -> anyhow::Result<()> {
        let service_account_email = "test@example.iam.gserviceaccount.com";
        let creator = Creator::new(TEST_PRIVATE_KEY_PEM, service_account_email.to_string())?;
        let uid = "user123";
        let now = 1700000000_u64;

        let token = creator.create(uid, now)?;

        // トークンクレームをデコードして検証
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = false;
        validation.set_required_spec_claims::<&str>(&[]);
        validation.set_audience(&["https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit"]);
        validation.set_issuer(&[service_account_email]);

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY_PEM.as_bytes())?;
        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(&token, &decoding_key, &validation)?;

        assert_eq!(token_data.claims.iss, service_account_email);
        assert_eq!(token_data.claims.sub, service_account_email);
        assert_eq!(
            token_data.claims.aud,
            "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit"
        );
        assert_eq!(token_data.claims.iat, now);
        assert_eq!(token_data.claims.exp, now + 3600);
        assert_eq!(token_data.claims.uid, uid);

        Ok(())
    }
}
