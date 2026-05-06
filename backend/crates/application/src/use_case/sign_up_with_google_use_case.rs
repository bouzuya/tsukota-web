use std::sync::Arc;

use domain::User;
use domain::UserCommand;
use domain::UserId;

use crate::error::ApplicationError;
use crate::error::GoogleUserError;
use crate::repository::GoogleUserMapRepository;
use crate::repository::UserRepository;
use crate::request::SignUpWithGoogleRequest;
use crate::response::SignUpWithGoogleResponse;

/// Google アカウントによるサインアップユースケース
///
/// Google sub が未登録なら新規 UserId を発行し、User 集約を作成する。
/// 既登録の場合は GoogleUserError::AlreadyRegistered
///
/// TODO: User イベント保存と GoogleUserMap 登録は現状順次実行で、
/// 片方だけ成功する事故を完全には防げない。Firestore 跨ぎトランザクション化を別途検討。
#[derive(Clone)]
pub struct SignUpWithGoogleUseCase {
    google_user_map_repository: Arc<dyn GoogleUserMapRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl SignUpWithGoogleUseCase {
    pub fn new(
        google_user_map_repository: Arc<dyn GoogleUserMapRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            google_user_map_repository,
            user_repository,
        }
    }

    #[tracing::instrument(name = "sign_up_with_google", skip(self))]
    pub async fn execute(
        &self,
        SignUpWithGoogleRequest { google_user_id }: SignUpWithGoogleRequest,
    ) -> Result<SignUpWithGoogleResponse, ApplicationError> {
        let existing = self
            .google_user_map_repository
            .find_user_id_by_google_user_id(&google_user_id)
            .await?;
        match existing {
            Some(_) => Err(ApplicationError::GoogleUser(
                GoogleUserError::AlreadyRegistered,
            )),
            None => {
                let user_id = UserId::generate();

                let user = User::new();
                let events = user
                    .handle_command(UserCommand::CreateUser { user_id })
                    .map_err(ApplicationError::User)?;

                self.user_repository.save_events(&user_id, events).await?;

                self.google_user_map_repository
                    .save(&google_user_id, &user_id)
                    .await?;

                Ok(SignUpWithGoogleResponse { user_id })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use domain::GoogleUserId;
    use domain::UserEvent;

    use super::*;

    struct InMemoryGoogleUserMap {
        entries: Mutex<Vec<(GoogleUserId, UserId)>>,
    }

    impl InMemoryGoogleUserMap {
        fn new() -> Self {
            Self {
                entries: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl GoogleUserMapRepository for InMemoryGoogleUserMap {
        async fn find_user_id_by_google_user_id(
            &self,
            google_user_id: &GoogleUserId,
        ) -> Result<Option<UserId>, ApplicationError> {
            let entries = self.entries.lock().expect("poisoned");
            Ok(entries
                .iter()
                .find(|(g, _)| g == google_user_id)
                .map(|(_, u)| *u))
        }

        async fn save(
            &self,
            google_user_id: &GoogleUserId,
            user_id: &UserId,
        ) -> Result<(), ApplicationError> {
            let mut entries = self.entries.lock().expect("poisoned");
            if entries.iter().any(|(g, _)| g == google_user_id) {
                return Err(ApplicationError::GoogleUser(
                    GoogleUserError::AlreadyRegistered,
                ));
            }
            entries.push((google_user_id.clone(), *user_id));
            Ok(())
        }
    }

    struct InMemoryUserRepo {
        events: Mutex<Vec<(UserId, Vec<UserEvent>)>>,
    }

    impl InMemoryUserRepo {
        fn new() -> Self {
            Self {
                events: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl UserRepository for InMemoryUserRepo {
        async fn load_events(
            &self,
            user_id: &UserId,
        ) -> Result<Vec<UserEvent>, ApplicationError> {
            let events = self.events.lock().expect("poisoned");
            Ok(events
                .iter()
                .find(|(u, _)| u == user_id)
                .map(|(_, e)| e.clone())
                .unwrap_or_default())
        }

        async fn save_events(
            &self,
            user_id: &UserId,
            new_events: Vec<UserEvent>,
        ) -> Result<(), ApplicationError> {
            let mut events = self.events.lock().expect("poisoned");
            events.push((*user_id, new_events));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_sign_up_creates_new_user_when_not_registered() -> anyhow::Result<()> {
        let map_repo: Arc<dyn GoogleUserMapRepository> = Arc::new(InMemoryGoogleUserMap::new());
        let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::new());
        let use_case = SignUpWithGoogleUseCase::new(map_repo.clone(), user_repo);

        let google_user_id: GoogleUserId = "google-sub-123".parse()?;
        let response = use_case
            .execute(SignUpWithGoogleRequest {
                google_user_id: google_user_id.clone(),
            })
            .await?;

        let resolved = map_repo
            .find_user_id_by_google_user_id(&google_user_id)
            .await?;
        assert_eq!(resolved, Some(response.user_id));
        Ok(())
    }

    #[tokio::test]
    async fn test_sign_up_fails_when_already_registered() -> anyhow::Result<()> {
        let map_repo: Arc<dyn GoogleUserMapRepository> = Arc::new(InMemoryGoogleUserMap::new());
        let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::new());
        let google_user_id: GoogleUserId = "google-sub-123".parse()?;
        let existing_user_id = UserId::generate();
        map_repo.save(&google_user_id, &existing_user_id).await?;

        let use_case = SignUpWithGoogleUseCase::new(map_repo, user_repo);
        let result = use_case
            .execute(SignUpWithGoogleRequest { google_user_id })
            .await;
        match result {
            Err(ApplicationError::GoogleUser(GoogleUserError::AlreadyRegistered)) => Ok(()),
            other => anyhow::bail!("Expected AlreadyRegistered error, got: {:?}", other),
        }
    }
}
