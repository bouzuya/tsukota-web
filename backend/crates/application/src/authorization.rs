use domain::UserId;

use crate::error::ApplicationError;
use crate::view::AccountView;

/// Verify that a user is an owner of an account
pub fn verify_owner(account: &AccountView, user_id: &UserId) -> Result<(), ApplicationError> {
    let user_id_str = user_id.to_string();
    if !account.owner_ids.contains(&user_id_str) {
        return Err(ApplicationError::Unauthorized(format!(
            "User {} is not an owner of account {}",
            user_id_str, account.id
        )));
    }
    Ok(())
}
