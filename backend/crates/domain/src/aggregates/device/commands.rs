use crate::DeviceId;
use crate::DeviceSecret;

/// デバイス集約に対するコマンド
#[derive(Clone, Debug, PartialEq)]
pub enum DeviceCommand {
    /// デバイスを作成する
    CreateDevice {
        device_id: DeviceId,
        device_secret: DeviceSecret,
    },
}
