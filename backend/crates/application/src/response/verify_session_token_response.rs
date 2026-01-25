/// セッショントークン検証レスポンス
#[derive(Clone, Debug)]
pub struct VerifySessionTokenResponse {
    /// トークンから取得したユーザー ID
    pub user_id: String,
}
