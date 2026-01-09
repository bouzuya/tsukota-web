use domain::account::UserId;

use crate::error::ApplicationError;
use crate::projection::AccountProjection;
use crate::view::AccountView;

/// Use case for listing all accounts owned by a user
pub struct ListAccountsUseCase<P: AccountProjection> {
    projection: P,
}

impl<P: AccountProjection> ListAccountsUseCase<P> {
    pub fn new(projection: P) -> Self {
        Self { projection }
    }

    pub async fn execute(&self, owner_id: &UserId) -> Result<Vec<AccountView>, ApplicationError> {
        self.projection.list_accounts(owner_id).await
    }
}
