use domain::Device;
use domain::DeviceCommand;
use domain::DeviceEvent;
use domain::DeviceId;
use domain::DeviceSecret;

/// テスト用のデバイスシークレット（32 バイト以上）
pub const TEST_SECRET: &str = "test-secret-value-32-bytes-long!";

/// デバイスを作成し、イベント適用済みの状態を返す
pub fn create_active_device() -> anyhow::Result<(Device, DeviceId, DeviceSecret)> {
    let mut device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = TEST_SECRET.parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret: device_secret.clone(),
    };
    let events = device.handle_command(command)?;
    apply_events(&mut device, &events);
    Ok((device, device_id, device_secret))
}

/// イベントを適用するヘルパー
pub fn apply_events(device: &mut Device, events: &[DeviceEvent]) {
    for event in events {
        device.apply_event(event);
    }
}
