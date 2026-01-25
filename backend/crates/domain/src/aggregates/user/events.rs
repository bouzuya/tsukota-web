/// ユーザーイベントの共通プロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEventCommonProps {
    pub at: String,
    pub id: String,
    pub user_id: String,
}

/// ユーザー集約に対するイベント
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum UserEvent {
    /// ユーザーが作成された
    #[serde(rename_all = "camelCase")]
    UserCreated {
        #[serde(flatten)]
        common: UserEventCommonProps,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_common_props() -> UserEventCommonProps {
        UserEventCommonProps {
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-123".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        }
    }

    #[test]
    fn test_user_created_serialization() -> anyhow::Result<()> {
        let event = UserEvent::UserCreated {
            common: create_common_props(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: UserEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "type": "userCreated",
            "userId": "550e8400-e29b-41d4-a716-446655440000"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }
}
