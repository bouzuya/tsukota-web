//! セッショントークンの JWT クレーム定義
//!
//! tsukota-web 独自のセッショントークンで使用する JWT クレーム。

/// セッショントークンの JWT クレーム
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SessionTokenClaims {
    /// Audience - トークンの対象サービス ("tsukota-web" 固定)
    pub aud: String,
    /// Expiration time - UNIX エポックからの秒数
    pub exp: u64,
    /// Issuer - トークン発行者 ("https://bouzuya.net/lab/tsukota" 固定)
    pub iss: String,
    /// Subject - ユーザー ID
    pub sub: String,
}

impl SessionTokenClaims {
    /// トークンの有効期限（12 時間 = 43200 秒）
    pub const EXPIRATION_SECONDS: u64 = 12 * 60 * 60;

    /// 固定の Audience 値
    pub const AUDIENCE: &'static str = "tsukota-web";

    /// 固定の Issuer 値
    pub const ISSUER: &'static str = "https://bouzuya.net/lab/tsukota";

    /// 新しい SessionTokenClaims インスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `user_id` - ユーザー ID (sub クレームに設定)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    ///
    /// # Returns
    ///
    /// 作成された SessionTokenClaims
    pub fn new(user_id: String, now: u64) -> Self {
        Self {
            aud: Self::AUDIENCE.to_owned(),
            exp: now + Self::EXPIRATION_SECONDS,
            iss: Self::ISSUER.to_owned(),
            sub: user_id,
        }
    }

    /// ユーザー ID を取得する
    pub fn user_id(&self) -> &str {
        &self.sub
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_claims_with_correct_values() -> anyhow::Result<()> {
        let user_id = "user123".to_owned();
        let now = 1700000000_u64;

        let claims = SessionTokenClaims::new(user_id.clone(), now);

        assert_eq!(claims.aud, "tsukota-web");
        assert_eq!(claims.iss, "https://bouzuya.net/lab/tsukota");
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.exp, now + 12 * 60 * 60);

        Ok(())
    }

    #[test]
    fn test_user_id_returns_sub() -> anyhow::Result<()> {
        let user_id = "test_user".to_owned();
        let claims = SessionTokenClaims::new(user_id.clone(), 1000);

        assert_eq!(claims.user_id(), &user_id);

        Ok(())
    }

    #[test]
    fn test_expiration_is_12_hours() -> anyhow::Result<()> {
        assert_eq!(SessionTokenClaims::EXPIRATION_SECONDS, 43200);

        Ok(())
    }
}
