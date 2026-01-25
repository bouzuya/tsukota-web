use std::sync::Arc;

use domain::Device;

use crate::error::ApplicationError;
use crate::repository::DeviceRepository;
use crate::repository::UserRepository;
use crate::request::CreateSessionTokenRequest;
use crate::response::CreateSessionTokenResponse;
use crate::session_token::SessionTokenCreator;

/// セッショントークン作成ユースケース
///
/// デバイス認証を行い、セッショントークンを発行する
#[derive(Clone)]
pub struct CreateSessionTokenUseCase {
    device_repository: Arc<dyn DeviceRepository>,
    creator: Arc<dyn SessionTokenCreator>,
    user_repository: Arc<dyn UserRepository>,
}

impl CreateSessionTokenUseCase {
    /// 新しいユースケースインスタンスを作成する
    pub fn new(
        device_repository: Arc<dyn DeviceRepository>,
        creator: Arc<dyn SessionTokenCreator>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            device_repository,
            creator,
            user_repository,
        }
    }

    /// セッショントークンを作成する
    ///
    /// # 処理フロー
    ///
    /// 1. デバイスドキュメントを取得
    /// 2. デバイスが存在する場合: bcrypt でシークレットを検証
    /// 3. デバイスが存在しない場合: 新しい UID を生成
    /// 4. セッショントークンを作成
    /// 5. デバイスドキュメントを保存
    /// 6. ユーザードキュメントを作成（存在しなければ）
    /// 7. セッショントークンを返す
    pub async fn execute(
        &self,
        CreateSessionTokenRequest {
            device_id,
            device_secret,
        }: CreateSessionTokenRequest,
    ) -> Result<CreateSessionTokenResponse, ApplicationError> {
        let device_id: domain::DeviceId = device_id
            .parse()
            .map_err(|_| ApplicationError::InvalidRequest("無効なデバイスID形式です".to_owned()))?;
        let device_secret: domain::DeviceSecret = device_secret.parse().map_err(|_| {
            ApplicationError::InvalidRequest("無効なデバイスシークレット形式です".to_owned())
        })?;

        // 1. デバイスドキュメントを取得
        let device_events = self.device_repository.load_events(&device_id).await?;
        let existing_device = Device::from_events(device_events);

        // 2 & 3. デバイスが存在する場合は検証、存在しない場合は新規作成
        let (device, events) = match existing_device {
            Device::Empty => {
                let mut device = Device::new();
                let events = device
                    .handle_command(domain::DeviceCommand::CreateDevice {
                        device_id,
                        device_secret,
                    })
                    .map_err(ApplicationError::Device)?;
                for event in &events {
                    device.apply_event(event);
                }
                (device, events)
            }
            Device::Active(device) => {
                if !device.verify(device_secret) {
                    return Err(ApplicationError::Unauthorized(
                        "デバイスシークレットが無効です".to_string(),
                    ));
                }
                (Device::Active(device), vec![])
            }
        };

        let device = match device {
            Device::Empty => unreachable!("デバイスは必ずアクティブ状態であるべきです"),
            Device::Active(active) => active,
        };

        // 4. セッショントークンを作成
        let now = self
            .creator
            .now()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let session_token = self
            .creator
            .create(&device.user_id().to_string(), now)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // 5. デバイスドキュメントを保存
        self.device_repository
            .save_events(&device.id(), events)
            .await?;

        // 6. ユーザードキュメントを作成（存在しなければ）
        let user_events = self.user_repository.load_events(&device.user_id()).await?;
        let existing_user = domain::User::from_events(user_events);
        match existing_user {
            domain::User::Empty => {
                let mut user = domain::User::new();
                let events = user
                    .handle_command(domain::UserCommand::CreateUser {
                        user_id: device.user_id(),
                    })
                    .map_err(ApplicationError::User)?;
                for event in &events {
                    user.apply_event(event);
                }
                let user = match &user {
                    domain::User::Empty => {
                        unreachable!("ユーザーは必ずアクティブ状態であるべきです")
                    }
                    domain::User::Active(user) => user,
                };
                self.user_repository.save_events(&user.id(), events).await?;
            }
            domain::User::Active(_) => {
                // 既にユーザーが存在する場合は何もしない
            }
        }

        // 7. セッショントークンを返す
        Ok(CreateSessionTokenResponse { session_token })
    }
}
