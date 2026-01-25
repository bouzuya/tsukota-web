/// セッショントークン作成レスポンス
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateSessionTokenResponse {
    /// Authorization ヘッダーに指定する Bearer トークン
    pub session_token: String,
}
