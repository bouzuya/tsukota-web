use std::sync::Arc;

use crate::error::ApplicationError;
use crate::request::VerifySessionTokenRequest;
use crate::response::VerifySessionTokenResponse;
use crate::session_token::SessionTokenVerifier;

/// セッショントークン検証ユースケース
///
/// セッショントークンを検証し、ユーザー ID を取得する
#[derive(Clone)]
pub struct VerifySessionTokenUseCase {
    verifier: Arc<dyn SessionTokenVerifier>,
}

impl VerifySessionTokenUseCase {
    /// 新しいユースケースインスタンスを作成する
    pub fn new(verifier: Arc<dyn SessionTokenVerifier>) -> Self {
        Self { verifier }
    }

    /// セッショントークンを検証する
    ///
    /// # 処理フロー
    ///
    /// 1. トークンを検証
    /// 2. ユーザー ID を取得
    pub fn execute(
        &self,
        VerifySessionTokenRequest { session_token }: VerifySessionTokenRequest,
    ) -> Result<VerifySessionTokenResponse, ApplicationError> {
        let user_id = self
            .verifier
            .verify(&session_token)
            .map_err(|e| ApplicationError::Unauthorized(e.to_string()))?;

        Ok(VerifySessionTokenResponse { user_id })
    }
}
