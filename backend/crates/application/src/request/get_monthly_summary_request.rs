/// 月別サマリー取得リクエスト
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GetMonthlySummaryRequest {
    pub account_id: String,
}
