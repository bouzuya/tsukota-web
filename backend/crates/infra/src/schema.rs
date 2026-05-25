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
    /// 月別支出合計 ("YYYY-MM" -> 0 未満の金額の合計、負の値で保存)
    pub expenses: BTreeMap<String, String>,
    pub id: String,
    /// 月別収入合計 ("YYYY-MM" -> 0 以上の金額の合計、非負値)
    pub incomes: BTreeMap<String, String>,
}

/// 取引クエリ用ドキュメント (`accounts/{account_id}/transactions/{transaction_id}`)
///
/// 取引参照系の高速化のため、書き込み時に同一トランザクションで更新する read model。
/// `date` は "YYYY-MM-DD" 形式で辞書順 = 時系列順。Firestore の `order_by` / `where`
/// で直接ソート・範囲検索が可能。
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryAccountTransactionDocumentData {
    pub account_id: String,
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub created_at: String,
    pub date: String,
    pub id: String,
    pub updated_at: String,
}
