/// デバイスイベントの共通プロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEventCommonProps {
    pub at: String,
    pub device_id: String,
    pub id: String,
}

/// デバイス集約に対するイベント
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DeviceEvent {
    /// デバイスが作成された
    #[serde(rename_all = "camelCase")]
    DeviceCreated {
        #[serde(flatten)]
        common: DeviceEventCommonProps,
        encrypted_secret: String,
        user_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_common_props() -> DeviceEventCommonProps {
        DeviceEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            device_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            id: "evt-123".to_string(),
        }
    }

    #[test]
    fn test_device_created_serialization() -> anyhow::Result<()> {
        let event = DeviceEvent::DeviceCreated {
            common: create_common_props(),
            encrypted_secret: "$2b$12$...".to_string(),
            user_id: "user-123".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: DeviceEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "at": "2024-01-01T00:00:00Z",
            "deviceId": "550e8400-e29b-41d4-a716-446655440000",
            "encryptedSecret": "$2b$12$...",
            "id": "evt-123",
            "type": "deviceCreated",
            "userId": "user-123"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }
}
