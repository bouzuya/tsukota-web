use domain::DeviceEvent;
use domain::DeviceEventCommonProps;

/// DeviceEvent::device_id() がイベントのデバイス ID を返すことを確認するテスト
#[test]
fn test_device_event_device_id() -> anyhow::Result<()> {
    let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let event = DeviceEvent::DeviceCreated {
        common: DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: device_uuid.to_string(),
            id: "evt-1".to_string(),
        },
        encrypted_secret: "$2b$12$...".to_string(),
        user_id: "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
    };

    assert_eq!(event.device_id(), device_uuid);
    Ok(())
}

/// DeviceEvent::device_id() が common.device_id と一致することを確認するテスト
#[test]
fn test_device_event_device_id_matches_common() -> anyhow::Result<()> {
    let device_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let common = DeviceEventCommonProps {
        at: "2024-06-15T12:00:00Z".to_string(),
        device_id: device_uuid.to_string(),
        id: "evt-abc".to_string(),
    };
    let event = DeviceEvent::DeviceCreated {
        common: common.clone(),
        encrypted_secret: "$2b$12$hash".to_string(),
        user_id: "user-xyz".to_string(),
    };

    assert_eq!(event.device_id(), &common.device_id);
    Ok(())
}
