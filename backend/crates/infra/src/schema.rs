//! Firestore ドキュメントのスキーマ定義

use std::collections::BTreeMap;

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

/// User 集約のイベントストリームドキュメント (`aggregates/user/event_streams/{user_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEventStreamDocumentData {
    pub id: String,
    pub updated_at: String,
}

/// Account ドキュメント (`accounts/{account_id}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAccountDocumentData {
    pub deleted_at: Option<String>,
    pub id: String,
    pub name: String,
    pub owners: Vec<String>,
}

/// User ドキュメント (`users/{user_id}`)
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryUserDocumentData {
    pub account_ids: Vec<String>,
    pub id: String,
}

/// Google sub と内部 UserId の対応ドキュメント (`google_user_map/{sub}`)
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryGoogleUserMapDocumentData {
    pub google_user_id: String,
    pub user_id: String,
}

/// 月別サマリードキュメント (`accounts/{account_id}/stats/monthly`)
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAccountMonthlySummaryDocumentData {
    pub id: String,
    /// 月別合計金額 ("YYYY-MM" -> 合計金額)
    pub totals: BTreeMap<String, String>,
}
