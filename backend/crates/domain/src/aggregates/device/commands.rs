use crate::value_objects::DeviceId;
use crate::value_objects::UserId;

/// デバイス集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum DeviceCommand {
    /// デバイスを作成する
    CreateDevice {
        device_id: DeviceId,
        encrypted_secret: String,
        user_id: UserId,
    },
}
