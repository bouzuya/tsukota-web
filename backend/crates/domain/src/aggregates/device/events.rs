/// デバイスイベントの共通プロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEventCommonProps {
    pub at: String,
    pub device_id: String,
    pub id: String,
}

/// デバイス集約に対するイベント
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DeviceEvent {
    /// デバイスが作成された
    #[serde(rename_all = "camelCase")]
    DeviceCreated {
        #[serde(flatten)]
        common: DeviceEventCommonProps,
        encrypted_secret: String,
        user_id: String,
    },
}

