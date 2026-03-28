use std::sync::Arc;

use domain::AccountId;

use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::projection::CategoryProjection;
use crate::request::ListCategoriesRequest;
use crate::response::ListCategoriesResponse;
use crate::view::PaginatedList;

/// Use case for listing all categories of an account
#[derive(Clone)]
pub struct ListCategoriesUseCase {
    account_projection: Arc<dyn AccountProjection>,
    category_projection: Arc<dyn CategoryProjection>,
}

impl ListCategoriesUseCase {
    pub fn new(
        account_projection: Arc<dyn AccountProjection>,
        category_projection: Arc<dyn CategoryProjection>,
    ) -> Self {
        Self {
            account_projection,
            category_projection,
        }
    }

    #[tracing::instrument(name = "list_categories", skip(self))]
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

        // Verify user is an owner of the account
        let owner_ids = self
            .account_projection
            .list_account_owner_ids(&account_id)
            .await?;
        crate::authorization::verify_owner(&account_id, &owner_ids, &domain_user_id)?;

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
