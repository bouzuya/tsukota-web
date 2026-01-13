use domain::account::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::CategoryProjection;
use crate::request::ListCategoriesRequest;
use crate::response::ListCategoriesResponse;
use crate::view::PaginatedList;

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
        user_id: &UserId,
        request: ListCategoriesRequest,
    ) -> Result<ListCategoriesResponse, ApplicationError> {
        let account_id: AccountId = request
            .account_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("Invalid account ID".into()))?;
        let domain_user_id = user_id.to_domain();

        // Get account to verify ownership
        let account = self
            .account_projection
            .get_account(&account_id)
            .await?
            .ok_or_else(|| {
                ApplicationError::AccountNotFound(format!("Account {} not found", account_id))
            })?;

        // Verify user is owner
        crate::authorization::verify_owner(&account, &domain_user_id)?;

        // Get categories
        let categories = self
            .category_projection
            .list_categories(&account_id)
            .await?;
        Ok(ListCategoriesResponse(PaginatedList {
            items: categories,
            next_cursor: None,
        }))
    }
}
