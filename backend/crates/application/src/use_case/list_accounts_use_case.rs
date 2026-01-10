use crate::UserId;
use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::request::ListAccountsRequest;
use crate::view::AccountView;

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
    ) -> Result<Vec<AccountView>, ApplicationError> {
        let owner_id = user_id.to_domain();
        self.projection.list_accounts(&owner_id).await
    }
}
