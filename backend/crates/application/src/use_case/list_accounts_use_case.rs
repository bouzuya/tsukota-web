use std::sync::Arc;

use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::request::ListAccountsRequest;
use crate::response::ListAccountsResponse;
use crate::view::PaginatedList;

/// Use case for listing all accounts owned by a user
#[derive(Clone)]
pub struct ListAccountsUseCase {
    projection: Arc<dyn AccountProjection>,
}

impl ListAccountsUseCase {
    pub fn new(projection: Arc<dyn AccountProjection>) -> Self {
        Self { projection }
    }

    pub async fn execute(
        &self,
        user_id: &UserId,
        _request: ListAccountsRequest,
    ) -> Result<ListAccountsResponse, ApplicationError> {
        let owner_id = user_id.to_domain();
        let accounts = self.projection.list_accounts(&owner_id).await?;
        Ok(ListAccountsResponse(PaginatedList {
            items: accounts,
            next_cursor: None,
        }))
    }
}
