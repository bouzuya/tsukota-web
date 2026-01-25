/// カスタムトークン作成リクエスト
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CreateCustomTokenRequest {
    /// デバイス識別子
    pub device_id: String,
    /// デバイスシークレット
    pub device_secret: String,
}
