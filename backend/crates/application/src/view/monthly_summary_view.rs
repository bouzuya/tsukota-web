use std::collections::BTreeMap;

/// 月別サマリーのビューモデル
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct MonthlySummaryView {
    pub account_id: String,
    /// 月別支出合計 ("YYYY-MM" -> 0 未満の金額の合計、負の値)
    pub expenses: BTreeMap<String, String>,
    /// 月別収入合計 ("YYYY-MM" -> 0 以上の金額の合計、非負値)
    pub incomes: BTreeMap<String, String>,
}
