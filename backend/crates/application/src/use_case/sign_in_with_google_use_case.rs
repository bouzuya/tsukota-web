use std::sync::Arc;

use crate::error::ApplicationError;
use crate::error::GoogleUserError;
use crate::repository::GoogleUserMapRepository;
use crate::request::SignInWithGoogleRequest;
use crate::response::SignInWithGoogleResponse;

/// Google アカウントによるサインインユースケース
///
/// Google sub に紐づく既存ユーザーを解決する。未登録の場合は GoogleUserError::NotRegistered
#[derive(Clone)]
pub struct SignInWithGoogleUseCase {
    google_user_map_repository: Arc<dyn GoogleUserMapRepository>,
}

impl SignInWithGoogleUseCase {
    pub fn new(google_user_map_repository: Arc<dyn GoogleUserMapRepository>) -> Self {
        Self {
            google_user_map_repository,
        }
    }

    #[tracing::instrument(name = "sign_in_with_google", skip(self))]
    pub async fn execute(
        &self,
        SignInWithGoogleRequest { google_user_id }: SignInWithGoogleRequest,
    ) -> Result<SignInWithGoogleResponse, ApplicationError> {
        let user_id = self
            .google_user_map_repository
            .find_user_id_by_google_user_id(&google_user_id)
            .await?
            .ok_or(ApplicationError::GoogleUser(GoogleUserError::NotRegistered))?;

        Ok(SignInWithGoogleResponse { user_id })
    }
}
