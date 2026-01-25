use super::commands::DeviceCommand;
use super::events::DeviceEvent;
use super::events::DeviceEventCommonProps;
use crate::value_objects::DeviceId;
use crate::value_objects::UserId;

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
    Active {
        /// デバイス ID
        id: DeviceId,
        /// 暗号化されたシークレット
        encrypted_secret: String,
        /// ユーザー ID
        user_id: UserId,
    },
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
                encrypted_secret,
                user_id,
            } => self.handle_create_device(device_id, encrypted_secret, user_id),
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
                *self = Device::Active {
                    id: common
                        .device_id
                        .parse()
                        .expect("Failed to parse device_id from event"),
                    encrypted_secret: encrypted_secret.clone(),
                    user_id: user_id
                        .parse()
                        .expect("Failed to parse user_id from event"),
                };
            }
        }
    }

    // コマンドハンドラの実装

    fn handle_create_device(
        &self,
        device_id: DeviceId,
        encrypted_secret: String,
        user_id: UserId,
    ) -> Result<Vec<DeviceEvent>, DeviceError> {
        if !matches!(self, Device::Empty) {
            return Err(DeviceError::DeviceAlreadyExists);
        }

        let common = Self::create_common_props(&device_id);
        Ok(vec![DeviceEvent::DeviceCreated {
            common,
            encrypted_secret,
            user_id: user_id.to_string(),
        }])
    }

    // ヘルパーメソッド

    fn create_common_props(device_id: &DeviceId) -> DeviceEventCommonProps {
        DeviceEventCommonProps {
            at: chrono::Utc::now().to_rfc3339(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_device() -> anyhow::Result<()> {
        let device = Device::new();
        let device_id = DeviceId::generate();
        let user_id = UserId::generate();
        let command = DeviceCommand::CreateDevice {
            device_id,
            encrypted_secret: "$2b$12$...".to_string(),
            user_id,
        };

        let events = device.handle_command(command)?;
        assert_eq!(events.len(), 1);

        match &events[0] {
            DeviceEvent::DeviceCreated {
                encrypted_secret,
                user_id: event_user_id,
                ..
            } => {
                assert_eq!(encrypted_secret, "$2b$12$...");
                assert_eq!(event_user_id, &user_id.to_string());
                Ok(())
            }
        }
    }

    #[test]
    fn test_device_from_events() -> anyhow::Result<()> {
        let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        let common = DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: device_uuid.to_string(),
            id: "evt-1".to_string(),
        };

        let events = vec![DeviceEvent::DeviceCreated {
            common,
            encrypted_secret: "$2b$12$...".to_string(),
            user_id: user_uuid.to_string(),
        }];

        let device = Device::from_events(events);
        match device {
            Device::Active {
                id,
                encrypted_secret,
                user_id,
            } => {
                assert_eq!(id.to_string(), device_uuid);
                assert_eq!(encrypted_secret, "$2b$12$...");
                assert_eq!(user_id.to_string(), user_uuid);
                Ok(())
            }
            Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
        }
    }

    #[test]
    fn test_device_already_exists() -> anyhow::Result<()> {
        let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
        let common = DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: device_uuid.to_string(),
            id: "evt-1".to_string(),
        };

        let mut device = Device::new();
        device.apply_event(&DeviceEvent::DeviceCreated {
            common,
            encrypted_secret: "$2b$12$...".to_string(),
            user_id: user_uuid.to_string(),
        });

        let new_device_id = DeviceId::generate();
        let new_user_id = UserId::generate();
        let command = DeviceCommand::CreateDevice {
            device_id: new_device_id,
            encrypted_secret: "$2b$12$new...".to_string(),
            user_id: new_user_id,
        };

        let result = device.handle_command(command);
        match result {
            Err(DeviceError::DeviceAlreadyExists) => Ok(()),
            Ok(_) => anyhow::bail!("Expected DeviceAlreadyExists error, but command succeeded"),
        }
    }
}
