/// セッショントークン検証リクエスト
#[derive(Clone, Debug)]
pub struct VerifySessionTokenRequest {
    /// 検証するセッショントークン
    pub session_token: String,
}
