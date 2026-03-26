use crate::DeviceCommand;
use crate::DeviceEvent;
use crate::DeviceEventCommonProps;
use crate::DeviceId;
use crate::DeviceSecret;
use crate::UserId;

/// デバイス集約のエラー
#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("Device already exists")]
    DeviceAlreadyExists,
}

/// デバイス集約
#[derive(Clone, Debug, PartialEq)]
pub enum Device {
    /// デバイス作成前の空の状態
    Empty,
    /// デバイス作成後のアクティブな状態
    Active(ActiveDevice),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActiveDevice {
    /// デバイス ID
    id: DeviceId,
    /// 暗号化されたシークレット (cost, salt を含む bcrypt ハッシュ)
    encrypted_secret: String,
    /// ユーザー ID
    user_id: UserId,
}

impl ActiveDevice {
    pub fn id(&self) -> DeviceId {
        self.id
    }

    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    pub fn verify(&self, device_secret: DeviceSecret) -> bool {
        bcrypt::verify(device_secret.to_string(), &self.encrypted_secret).unwrap_or(false)
    }
}

impl Default for Device {
    fn default() -> Self {
        Self::new()
    }
}

impl Device {
    /// 新しい空の集約を作成
    pub fn new() -> Self {
        Self::Empty
    }

    /// イベントストリームから集約を再構築
    pub fn from_events(events: Vec<DeviceEvent>) -> Self {
        let mut device = Self::new();
        for event in events {
            device.apply_event(&event);
        }
        device
    }

    /// コマンドを処理してイベントを生成
    pub fn handle_command(&self, command: DeviceCommand) -> Result<Vec<DeviceEvent>, DeviceError> {
        match command {
            DeviceCommand::CreateDevice {
                device_id,
                device_secret,
            } => self.handle_create_device(device_id, device_secret),
        }
    }

    /// イベントを適用して状態を更新
    pub fn apply_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::DeviceCreated {
                common,
                encrypted_secret,
                user_id,
            } => {
                *self = Device::Active(ActiveDevice {
                    id: common
                        .device_id
                        .parse()
                        .expect("Failed to parse device_id from event"),
                    encrypted_secret: encrypted_secret.clone(),
                    user_id: user_id.parse().expect("Failed to parse user_id from event"),
                });
            }
        }
    }

    // コマンドハンドラの実装

    fn handle_create_device(
        &self,
        device_id: DeviceId,
        device_secret: DeviceSecret,
    ) -> Result<Vec<DeviceEvent>, DeviceError> {
        if !matches!(self, Device::Empty) {
            return Err(DeviceError::DeviceAlreadyExists);
        }

        // FIXME: unwrap
        let encrypted_secret =
            bcrypt::hash(device_secret.to_string(), bcrypt::DEFAULT_COST).unwrap();

        let user_id = UserId::generate();

        let common = Self::create_common_props(&device_id);
        Ok(vec![DeviceEvent::DeviceCreated {
            common,
            encrypted_secret: encrypted_secret.to_string(),
            user_id: user_id.to_string(),
        }])
    }

    // ヘルパーメソッド

    fn create_common_props(device_id: &DeviceId) -> DeviceEventCommonProps {
        DeviceEventCommonProps {
            at: date_time::DateTime::now().to_string(),
            device_id: device_id.to_string(),
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

// DeviceEvent にヘルパーメソッドを追加
impl DeviceEvent {
    pub fn device_id(&self) -> &String {
        match self {
            DeviceEvent::DeviceCreated { common, .. } => &common.device_id,
        }
    }
}
