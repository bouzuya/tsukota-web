use domain::account::AccountId;
use domain::account::UserId;

use crate::error::ApplicationError;
use crate::error::Result;
use crate::projection::AccountProjection;
use crate::projection::CategoryProjection;
use crate::view::CategoryView;

/// Use case for listing all categories of an account
pub struct ListCategoriesUseCase<A: AccountProjection, C: CategoryProjection> {
    account_projection: A,
    category_projection: C,
}

impl<A: AccountProjection, C: CategoryProjection> ListCategoriesUseCase<A, C> {
    pub fn new(account_projection: A, category_projection: C) -> Self {
        Self {
            account_projection,
            category_projection,
        }
    }

    pub async fn execute(
        &self,
        account_id: &AccountId,
        user_id: &UserId,
    ) -> Result<Vec<CategoryView>> {
        // Get account to verify ownership
        let account = self
            .account_projection
            .get_account(account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        crate::authorization::verify_owner(&account, user_id)?;

        // Get categories
        self.category_projection.list_categories(account_id).await
    }
}
