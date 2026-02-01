//! Firestore ドキュメントのスキーマ定義

/// Account 集約のイベントストリームドキュメント (`aggregates/account/event_streams/{account_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEventStreamDocumentData {
    pub id: String,
    pub last_event_id: String,
    pub owners: Vec<String>,
    pub protocol_version: u32,
    pub updated_at: String,
}

/// Device 集約のイベントストリームドキュメント (`aggregates/device/event_streams/{device_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEventStreamDocumentData {
    pub id: String,
    pub updated_at: String,
}

/// User 集約のイベントストリームドキュメント (`aggregates/user/event_streams/{user_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEventStreamDocumentData {
    pub id: String,
    pub updated_at: String,
}

/// Device ドキュメント (`devices/{device_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDeviceDocumentData {
    pub encrypted_secret: String,
    pub id: String,
    pub uid: String,
}

/// User ドキュメント (`users/{user_id}`)
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryUserDocumentData {
    pub account_ids: Vec<String>,
    pub id: String,
}
