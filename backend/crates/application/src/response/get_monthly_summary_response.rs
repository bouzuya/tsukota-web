use crate::view::MonthlySummaryView;

/// 月別サマリー取得レスポンス
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct GetMonthlySummaryResponse(pub MonthlySummaryView);
