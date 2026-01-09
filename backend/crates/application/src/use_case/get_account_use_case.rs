use domain::account::AccountId;
use domain::account::UserId;

use crate::authorization::verify_owner;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::view::AccountView;

/// Use case for getting account details
pub struct GetAccountUseCase<P: AccountProjection> {
    projection: P,
}

impl<P: AccountProjection> GetAccountUseCase<P> {
    pub fn new(projection: P) -> Self {
        Self { projection }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        user_id: &UserId,
    ) -> Result<AccountView, ApplicationError> {
        // Get account from projection
        let account = self
            .projection
            .get_account(account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        verify_owner(&account, user_id)?;

        Ok(account)
    }
}
