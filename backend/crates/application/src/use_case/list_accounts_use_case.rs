use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::request::ListAccountsRequest;
use crate::response::ListAccountsResponse;
use crate::view::PaginatedList;

/// Use case for listing all accounts owned by a user
pub struct ListAccountsUseCase<P: AccountProjection> {
    projection: P,
}

impl<P: AccountProjection> ListAccountsUseCase<P> {
    pub fn new(projection: P) -> Self {
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
