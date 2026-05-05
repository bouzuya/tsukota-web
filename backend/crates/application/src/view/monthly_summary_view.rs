use std::collections::BTreeMap;

/// 月別サマリーのビューモデル
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MonthlySummaryView {
    pub account_id: String,
    /// 月別合計金額 ("YYYY-MM" -> 合計金額)
    pub totals: BTreeMap<String, String>,
}
