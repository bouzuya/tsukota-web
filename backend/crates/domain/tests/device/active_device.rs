use crate::helper;
use domain::DeviceSecret;

/// ActiveDevice::id() が正しいデバイス ID を返すことを確認するテスト
#[test]
fn test_active_device_id() -> anyhow::Result<()> {
    let (device, device_id, _) = helper::create_active_device()?;
    match device {
        domain::Device::Active(active) => {
            assert_eq!(active.id(), device_id);
            Ok(())
        }
        domain::Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

/// ActiveDevice::user_id() が有効なユーザー ID を返すことを確認するテスト
#[test]
fn test_active_device_user_id() -> anyhow::Result<()> {
    let (device, _, _) = helper::create_active_device()?;
    match device {
        domain::Device::Active(active) => {
            let user_id = active.user_id();
            // UserId が文字列に変換できることを確認
            assert!(!user_id.to_string().is_empty());
            Ok(())
        }
        domain::Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

/// ActiveDevice::user_id() が生成されるたびに異なることを確認するテスト
#[test]
fn test_active_device_user_id_is_unique() -> anyhow::Result<()> {
    let (device1, _, _) = helper::create_active_device()?;
    let (device2, _, _) = helper::create_active_device()?;
    match (device1, device2) {
        (domain::Device::Active(active1), domain::Device::Active(active2)) => {
            // それぞれ別の CreateDevice コマンドで生成したので user_id は異なる
            assert_ne!(active1.user_id(), active2.user_id());
            Ok(())
        }
        _ => anyhow::bail!("Expected two Active devices"),
    }
}

/// ActiveDevice::verify() が正しいシークレットを受け付けることを確認するテスト
#[test]
fn test_active_device_verify_correct_secret() -> anyhow::Result<()> {
    let (device, _, _) = helper::create_active_device()?;
    let device_secret: DeviceSecret = helper::TEST_SECRET.parse()?;
    match device {
        domain::Device::Active(active) => {
            assert!(active.verify(device_secret));
            Ok(())
        }
        domain::Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}

/// ActiveDevice::verify() が誤ったシークレットを拒否することを確認するテスト
#[test]
fn test_active_device_verify_incorrect_secret() -> anyhow::Result<()> {
    let (device, _, _) = helper::create_active_device()?;
    let wrong_secret: DeviceSecret = "wrong-secret-value-32-bytes-long!".parse()?;
    match device {
        domain::Device::Active(active) => {
            assert!(!active.verify(wrong_secret));
            Ok(())
        }
        domain::Device::Empty => anyhow::bail!("Expected Active device, got Empty"),
    }
}
