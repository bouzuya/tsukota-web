use domain::Device;

use crate::error::ApplicationError;
use crate::repository::DeviceRepository;
use crate::repository::UserRepository;
use crate::request::CreateCustomTokenRequest;
use crate::response::CreateCustomTokenResponse;
use crate::token_signer::TokenSigner;

/// カスタムトークン作成ユースケース
///
/// デバイス認証を行い、Firebase カスタムトークンを発行する
pub struct CreateCustomTokenUseCase<DR, S: TokenSigner, UR: UserRepository> {
    device_repository: DR,
    signer: S,
    user_repository: UR,
}

impl<DR: DeviceRepository, S: TokenSigner, UR: UserRepository> CreateCustomTokenUseCase<DR, S, UR> {
    /// 新しいユースケースインスタンスを作成する
    pub fn new(device_repository: DR, signer: S, user_repository: UR) -> Self {
        Self {
            device_repository,
            signer,
            user_repository,
        }
    }

    /// カスタムトークンを作成する
    ///
    /// # 処理フロー
    ///
    /// 1. デバイスドキュメントを取得
    /// 2. デバイスが存在する場合: bcrypt でシークレットを検証
    /// 3. デバイスが存在しない場合: 新しい UID を生成
    /// 4. カスタムトークンを署名
    /// 5. デバイスドキュメントを保存
    /// 6. ユーザードキュメントを作成（存在しなければ）
    /// 7. カスタムトークンを返す
    pub async fn execute(
        &self,
        CreateCustomTokenRequest {
            device_id,
            device_secret,
        }: CreateCustomTokenRequest,
    ) -> Result<CreateCustomTokenResponse, ApplicationError> {
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

        // 4. カスタムトークンを署名
        let now = self
            .signer
            .now()
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let custom_token = self
            .signer
            .sign(&device.user_id().to_string(), now)
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

        // 7. カスタムトークンを返す
        Ok(CreateCustomTokenResponse { custom_token })
    }
}
