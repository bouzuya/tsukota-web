use domain::Device;
use domain::DeviceEvent;
use domain::DeviceEventCommonProps;

/// apply_event で DeviceCreated を適用すると Active になることを確認するテスト
#[test]
fn test_apply_event_device_created() -> anyhow::Result<()> {
    let mut device = Device::new();
    let device_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let event = DeviceEvent::DeviceCreated {
        common: DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: device_uuid.to_string(),
            id: "evt-1".to_string(),
        },
        encrypted_secret: "$2b$12$...".to_string(),
        user_id: user_uuid.to_string(),
    };

    device.apply_event(&event);

    match device {
        Device::Active(active) => {
            assert_eq!(active.id().to_string(), device_uuid);
            assert_eq!(active.user_id().to_string(), user_uuid);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

/// Empty 状態に apply_event を複数回適用した場合でも最後のイベントが反映されるテスト
#[test]
fn test_apply_event_overwrites_state() -> anyhow::Result<()> {
    let device_uuid1 = "550e8400-e29b-41d4-a716-446655440000";
    let device_uuid2 = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";

    let mut device = Device::new();
    device.apply_event(&DeviceEvent::DeviceCreated {
        common: DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: device_uuid1.to_string(),
            id: "evt-1".to_string(),
        },
        encrypted_secret: "$2b$12$hash1".to_string(),
        user_id: user_uuid.to_string(),
    });
    device.apply_event(&DeviceEvent::DeviceCreated {
        common: DeviceEventCommonProps {
            at: "2024-01-02T00:00:00Z".to_string(),
            device_id: device_uuid2.to_string(),
            id: "evt-2".to_string(),
        },
        encrypted_secret: "$2b$12$hash2".to_string(),
        user_id: user_uuid.to_string(),
    });

    match device {
        Device::Active(active) => {
            assert_eq!(active.id().to_string(), device_uuid2);
            Ok(())
        }
        Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}
