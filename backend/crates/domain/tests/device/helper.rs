use domain::Device;
use domain::DeviceCommand;
use domain::DeviceEvent;
use domain::DeviceId;
use domain::DeviceSecret;

/// デバイスを作成し、イベント適用済みの状態を返す
pub fn create_active_device() -> anyhow::Result<(Device, DeviceId)> {
    let mut device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = "test-secret-value-32-bytes-long!".parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret,
    };
    let events = device.handle_command(command)?;
    apply_events(&mut device, &events);
    Ok((device, device_id))
}

/// イベントを適用するヘルパー
pub fn apply_events(device: &mut Device, events: &[DeviceEvent]) {
    for event in events {
        device.apply_event(event);
    }
}
