use application::SessionTokenCreator;
use application::SessionTokenVerifier;

use super::claims::SessionTokenClaims;

/// トークン作成エラー
#[derive(Debug)]
pub enum PemSessionTokenCreatorError {
    /// JWT エンコードに失敗
    JwtEncodingError(jsonwebtoken::errors::Error),
}

impl std::fmt::Display for PemSessionTokenCreatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PemSessionTokenCreatorError::JwtEncodingError(e) => {
                write!(f, "JWT encoding error: {}", e)
            }
        }
    }
}

impl std::error::Error for PemSessionTokenCreatorError {}

/// セッショントークン作成器
#[derive(Clone)]
pub struct PemSessionTokenCreator {
    encoding_key: jsonwebtoken::EncodingKey,
}

impl PemSessionTokenCreator {
    /// 新しいインスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - RSA 秘密鍵 (PEM 形式)
    pub fn new(private_key_pem: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
        Ok(Self { encoding_key })
    }

    /// 指定されたユーザーのセッショントークンを作成する
    ///
    /// # Arguments
    ///
    /// * `user_id` - ユーザー識別子
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    ///
    /// # Returns
    ///
    /// JWT 形式のセッショントークン
    pub fn create(&self, user_id: &str, now: u64) -> Result<String, PemSessionTokenCreatorError> {
        // クレームを作成
        let claims = SessionTokenClaims::new(user_id.to_owned(), now);

        // RS256 アルゴリズムでヘッダーを作成
        let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

        // トークンをエンコード
        jsonwebtoken::encode(&header, &claims, &self.encoding_key)
            .map_err(PemSessionTokenCreatorError::JwtEncodingError)
    }
}

#[async_trait::async_trait]
impl SessionTokenCreator for PemSessionTokenCreator {
    async fn create(
        &self,
        user_id: &str,
        now: u64,
    ) -> Result<String, application::session_token::CreatorError> {
        self.create(user_id, now)
            .map_err(|e| Box::new(e) as application::session_token::CreatorError)
    }
}

/// トークン検証エラー
#[derive(Debug)]
pub enum PemSessionTokenVerifierError {
    /// JWT デコードに失敗
    JwtDecodeError(jsonwebtoken::errors::Error),
}

impl std::fmt::Display for PemSessionTokenVerifierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PemSessionTokenVerifierError::JwtDecodeError(e) => {
                write!(f, "JWT decode error: {}", e)
            }
        }
    }
}

impl std::error::Error for PemSessionTokenVerifierError {}

/// JWT トークンの検証器
#[derive(Clone)]
pub struct PemSessionTokenVerifier {
    decoding_key: jsonwebtoken::DecodingKey,
}

impl PemSessionTokenVerifier {
    /// 新しいインスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `private_key_pem` - RSA 秘密鍵 (PEM 形式)
    pub fn new(private_key_pem: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        // 秘密鍵から公開鍵コンポーネントを抽出してデコード用キーを作成
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(
            Self::extract_public_key(private_key_pem)?.as_bytes(),
        )?;
        Ok(Self { decoding_key })
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

    /// トークンを検証してユーザー ID を取得する
    ///
    /// exp, aud, iss を検証する
    pub fn verify(&self, token: &str) -> Result<String, PemSessionTokenVerifierError> {
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[SessionTokenClaims::AUDIENCE]);
        validation.set_issuer(&[SessionTokenClaims::ISSUER]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "sub"]);
        validation.validate_exp = true;

        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(token, &self.decoding_key, &validation)
                .map_err(PemSessionTokenVerifierError::JwtDecodeError)?;

        Ok(token_data.claims.user_id().to_owned())
    }
}

#[async_trait::async_trait]
impl SessionTokenVerifier for PemSessionTokenVerifier {
    async fn verify(
        &self,
        token: &str,
    ) -> Result<String, application::session_token::VerifierError> {
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
    fn test_create_success() -> anyhow::Result<()> {
        let creator = PemSessionTokenCreator::new(TEST_PRIVATE_KEY_PEM)?;
        let user_id = "user123";
        let now = 1700000000_u64;

        let token = creator.create(user_id, now)?;

        // トークンクレームをデコードして検証
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = false;
        validation.set_audience(&[SessionTokenClaims::AUDIENCE]);
        validation.set_issuer(&[SessionTokenClaims::ISSUER]);
        validation.set_required_spec_claims(&["aud", "exp", "iss", "sub"]);
        // exp はテストコードなので意図的に含めていない
        // validation.validate_exp = true;

        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY_PEM.as_bytes())?;
        let token_data =
            jsonwebtoken::decode::<SessionTokenClaims>(&token, &decoding_key, &validation)?;

        assert_eq!(token_data.claims.iss, SessionTokenClaims::ISSUER);
        assert_eq!(token_data.claims.aud, SessionTokenClaims::AUDIENCE);
        assert_eq!(token_data.claims.sub, user_id);
        assert_eq!(
            token_data.claims.exp,
            now + SessionTokenClaims::EXPIRATION_SECONDS
        );

        Ok(())
    }

    #[test]
    fn test_create_and_verify() -> anyhow::Result<()> {
        let creator = PemSessionTokenCreator::new(TEST_PRIVATE_KEY_PEM)?;
        let verifier = PemSessionTokenVerifier::new(TEST_PRIVATE_KEY_PEM)?;

        let user_id = "test_user_123";
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let token = creator.create(user_id, now)?;
        let verified_user_id = verifier.verify(&token)?;

        assert_eq!(verified_user_id, user_id);

        Ok(())
    }
}
