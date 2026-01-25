/// カスタムトークン作成レスポンス
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateCustomTokenResponse {
    /// Authorization ヘッダーに指定する Bearer トークン
    pub custom_token: String,
}
