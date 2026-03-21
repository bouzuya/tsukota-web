use crate::helper;
use domain::Device;
use domain::DeviceCommand;
use domain::DeviceError;
use domain::DeviceEvent;
use domain::DeviceEventCommonProps;
use domain::DeviceId;
use domain::DeviceSecret;
use domain::UserId;

/// Device::new() が Empty を返すことを確認するテスト
#[test]
fn test_device_new() -> anyhow::Result<()> {
    let device = Device::new();
    assert_eq!(device, Device::Empty);
    Ok(())
}

/// CreateDevice コマンド成功のテスト
#[test]
fn test_create_device() -> anyhow::Result<()> {
    let device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = helper::TEST_SECRET.parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret,
    };

    let events = device.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        DeviceEvent::DeviceCreated {
            common,
            encrypted_secret,
            user_id,
        } => {
            assert_eq!(common.device_id, device_id.to_string());
            // bcrypt ハッシュ形式であることを確認
            assert!(encrypted_secret.starts_with("$2b$"));
            // user_id が有効な UUID であることを確認
            assert!(user_id.parse::<UserId>().is_ok());
            Ok(())
        }
    }
}

/// 既存デバイスへの CreateDevice コマンドがエラーになることを確認するテスト
#[test]
fn test_create_device_already_exists() -> anyhow::Result<()> {
    let (device, _, _) = helper::create_active_device()?;
    let new_device_id = DeviceId::generate();
    let new_device_secret: DeviceSecret = "new-secret-value-32-bytes-long!!".parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id: new_device_id,
        device_secret: new_device_secret,
    };

    match device.handle_command(command) {
        Err(DeviceError::DeviceAlreadyExists) => Ok(()),
        Ok(_) => anyhow::bail!("Expected DeviceAlreadyExists error, but command succeeded"),
    }
}

/// イベントなしで from_events すると Empty になることを確認するテスト
#[test]
fn test_device_from_empty_events() -> anyhow::Result<()> {
    let device = Device::from_events(vec![]);
    assert_eq!(device, Device::Empty);
    Ok(())
}

/// from_events でアクティブなデバイスを再構築するテスト
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
        Device::Active(active) => {
            assert_eq!(active.id().to_string(), device_uuid);
            assert_eq!(active.user_id().to_string(), user_uuid);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

/// handle_command で生成したイベントから from_events で再構築するテスト
#[test]
fn test_device_from_events_roundtrip() -> anyhow::Result<()> {
    let device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = helper::TEST_SECRET.parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret,
    };
    let events = device.handle_command(command)?;

    let reconstructed = Device::from_events(events);
    match reconstructed {
        Device::Active(active) => {
            assert_eq!(active.id(), device_id);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}
