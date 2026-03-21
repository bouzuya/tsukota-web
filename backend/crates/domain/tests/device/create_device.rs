use crate::helper;
use domain::Device;
use domain::DeviceCommand;
use domain::DeviceError;
use domain::DeviceEvent;
use domain::DeviceEventCommonProps;
use domain::DeviceId;
use domain::DeviceSecret;
use domain::UserId;

#[test]
fn test_device_default() -> anyhow::Result<()> {
    // Device::default() は空の集約を返すことを確認
    let device = Device::default();
    assert_eq!(device, Device::Empty);
    Ok(())
}

#[test]
fn test_device_from_empty_events() -> anyhow::Result<()> {
    // 空のイベントリストから再構築すると Empty になることを確認
    let device = Device::from_events(vec![]);
    assert_eq!(device, Device::Empty);
    Ok(())
}

#[test]
fn test_create_device() -> anyhow::Result<()> {
    let device = Device::new();
    let device_id = DeviceId::generate();
    // 32 バイト以上のシークレット
    let device_secret: DeviceSecret = "test-secret-value-32-bytes-long!".parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret: device_secret.clone(),
    };

    let events = device.handle_command(command)?;
    assert_eq!(events.len(), 1);

    // イベントを適用して user_id が有効な UUID であることを確認
    match &events[0] {
        DeviceEvent::DeviceCreated { user_id, .. } => {
            assert!(user_id.parse::<UserId>().is_ok());
        }
    }

    // アクティブ状態に遷移し、verify() で正しいシークレットを検証できることを確認
    let active_device = Device::from_events(events);
    match active_device {
        Device::Active(ref ad) => {
            assert!(ad.verify(device_secret));
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_create_device_common_props() -> anyhow::Result<()> {
    // 生成されたイベントの common.device_id がコマンドの device_id と一致することを確認
    let device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = "test-secret-value-32-bytes-long!".parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret,
    };

    let events = device.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        DeviceEvent::DeviceCreated { common, .. } => {
            assert_eq!(common.device_id, device_id.to_string());
            // event id は UUID 形式であることを確認
            assert!(common.id.parse::<DeviceId>().is_ok());
            Ok(())
        }
    }
}

#[test]
fn test_create_device_generates_unique_user_ids() -> anyhow::Result<()> {
    // 2 回のデバイス作成でユニークな user_id が生成されることを確認
    let device1 = Device::new();
    let device2 = Device::new();
    let secret: DeviceSecret = "test-secret-value-32-bytes-long!".parse()?;

    let events1 = device1.handle_command(DeviceCommand::CreateDevice {
        device_id: DeviceId::generate(),
        device_secret: secret.clone(),
    })?;
    let events2 = device2.handle_command(DeviceCommand::CreateDevice {
        device_id: DeviceId::generate(),
        device_secret: secret,
    })?;

    let user_id1 = match &events1[0] {
        DeviceEvent::DeviceCreated { user_id, .. } => user_id.clone(),
    };
    let user_id2 = match &events2[0] {
        DeviceEvent::DeviceCreated { user_id, .. } => user_id.clone(),
    };

    assert_ne!(user_id1, user_id2);
    Ok(())
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
        Device::Active(ref ad) => {
            assert_eq!(ad.id().to_string(), device_uuid);
            assert_eq!(ad.user_id().to_string(), user_uuid);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_device_already_exists() -> anyhow::Result<()> {
    let (device, _) = helper::create_active_device()?;

    let new_device_id = DeviceId::generate();
    // 32 バイト以上のシークレット
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

#[test]
fn test_device_verify_correct_secret() -> anyhow::Result<()> {
    // 正しいシークレットで verify() が true を返すことを確認
    let device = Device::new();
    let device_id = DeviceId::generate();
    let secret_value = "test-secret-value-32-bytes-long!";
    let device_secret: DeviceSecret = secret_value.parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret: device_secret.clone(),
    };

    let events = device.handle_command(command)?;
    let active_device = Device::from_events(events);

    match active_device {
        Device::Active(ref ad) => {
            assert!(ad.verify(device_secret));
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_device_verify_wrong_secret() -> anyhow::Result<()> {
    // 誤ったシークレットで verify() が false を返すことを確認
    let device = Device::new();
    let device_id = DeviceId::generate();
    let device_secret: DeviceSecret = "test-secret-value-32-bytes-long!".parse()?;
    let command = DeviceCommand::CreateDevice {
        device_id,
        device_secret,
    };

    let events = device.handle_command(command)?;
    let active_device = Device::from_events(events);
    let wrong_secret: DeviceSecret = "wrong-secret-value-32-bytes-long".parse()?;

    match active_device {
        Device::Active(ref ad) => {
            assert!(!ad.verify(wrong_secret));
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_device_id_getter() -> anyhow::Result<()> {
    // id() が正しいデバイス ID を返すことを確認
    let (device, device_id) = helper::create_active_device()?;

    match device {
        Device::Active(ref ad) => {
            assert_eq!(ad.id(), device_id);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_device_user_id_getter() -> anyhow::Result<()> {
    // user_id() が正しいユーザー ID を返すことを確認
    let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let common = DeviceEventCommonProps {
        at: "2024-01-01T00:00:00Z".to_string(),
        device_id: device_uuid.to_string(),
        id: "evt-1".to_string(),
    };

    let device = Device::from_events(vec![DeviceEvent::DeviceCreated {
        common,
        encrypted_secret: "$2b$12$...".to_string(),
        user_id: user_uuid.to_string(),
    }]);

    match device {
        Device::Active(ref ad) => {
            assert_eq!(ad.user_id().to_string(), user_uuid);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

#[test]
fn test_device_event_device_id() -> anyhow::Result<()> {
    // DeviceEvent::device_id() ヘルパーが正しいデバイス ID を返すことを確認
    let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let common = DeviceEventCommonProps {
        at: "2024-01-01T00:00:00Z".to_string(),
        device_id: device_uuid.to_string(),
        id: "evt-1".to_string(),
    };

    let event = DeviceEvent::DeviceCreated {
        common,
        encrypted_secret: "$2b$12$...".to_string(),
        user_id: "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
    };

    assert_eq!(event.device_id(), device_uuid);
    Ok(())
}
