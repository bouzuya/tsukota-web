use std::sync::Arc;

use domain::AccountId;

use crate::UserId;
use crate::authorization::verify_owner;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::request::GetAccountRequest;
use crate::response::GetAccountResponse;

/// Use case for getting account details
#[derive(Clone)]
pub struct GetAccountUseCase {
    projection: Arc<dyn AccountProjection>,
}

impl GetAccountUseCase {
    pub fn new(projection: Arc<dyn AccountProjection>) -> Self {
        Self { projection }
    }

    pub async fn execute(
        &self,
        user_id: &UserId,
        request: GetAccountRequest,
    ) -> Result<GetAccountResponse, ApplicationError> {
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".into()))?;
        let domain_user_id = user_id.to_domain();

        // Get account from projection
        let account = self
            .projection
            .get_account(&account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        verify_owner(&account, &domain_user_id)?;

        Ok(GetAccountResponse(account))
    }
}
