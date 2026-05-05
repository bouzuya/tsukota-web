use std::sync::Arc;

use domain::AccountId;

use crate::UserId;
use crate::authorization;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::MonthlySummaryProjection;
use crate::request::GetMonthlySummaryRequest;
use crate::response::GetMonthlySummaryResponse;

/// 月別サマリー取得ユースケース
#[derive(Clone)]
pub struct GetMonthlySummaryUseCase {
    account_projection: Arc<dyn AccountProjection>,
    monthly_summary_projection: Arc<dyn MonthlySummaryProjection>,
}

impl GetMonthlySummaryUseCase {
    pub fn new(
        account_projection: Arc<dyn AccountProjection>,
        monthly_summary_projection: Arc<dyn MonthlySummaryProjection>,
    ) -> Self {
        Self {
            account_projection,
            monthly_summary_projection,
        }
    }

    #[tracing::instrument(name = "get_monthly_summary", skip(self))]
    pub async fn execute(
        &self,
        user_id: &UserId,
        request: GetMonthlySummaryRequest,
    ) -> Result<GetMonthlySummaryResponse, ApplicationError> {
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".into()))?;
        let domain_user_id = user_id.to_domain();

        // オーナー確認
        let owner_ids = self
            .account_projection
            .list_account_owner_ids(&account_id)
            .await?;
        authorization::verify_owner(&account_id, &owner_ids, &domain_user_id)?;

        // 月別サマリー取得
        let summary = self
            .monthly_summary_projection
            .get_monthly_summary(&account_id)
            .await?;

        Ok(GetMonthlySummaryResponse(summary))
    }
}
