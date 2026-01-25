/// イベントの共通プロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountEventCommonProps {
    pub account_id: String,
    pub at: String,
    pub id: String,
    pub protocol_version: u32,
}

/// 取引のプロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionProps {
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}

/// アカウント集約に対するイベント
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AccountEvent {
    /// アカウントが作成された
    #[serde(rename_all = "camelCase")]
    AccountCreated {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        name: String,
        owners: Vec<String>,
    },

    /// アカウントが削除された
    #[serde(rename_all = "camelCase")]
    AccountDeleted {
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// アカウント名が変更された
    #[serde(rename_all = "camelCase")]
    AccountUpdated {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        name: String,
    },

    /// カテゴリが追加された
    #[serde(rename_all = "camelCase")]
    CategoryAdded {
        category_id: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
        name: String,
    },

    /// カテゴリが削除された
    #[serde(rename_all = "camelCase")]
    CategoryDeleted {
        category_id: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// カテゴリ名が変更された
    #[serde(rename_all = "camelCase")]
    CategoryUpdated {
        category_id: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
        name: String,
    },

    /// オーナーが追加された
    #[serde(rename_all = "camelCase")]
    OwnerAdded {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        owner: String,
    },

    /// オーナーが削除された
    #[serde(rename_all = "camelCase")]
    OwnerRemoved {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        owner: String,
    },

    /// 取引が追加された
    #[serde(rename_all = "camelCase")]
    TransactionAdded {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        #[serde(flatten)]
        props: TransactionProps,
        transaction_id: String,
    },

    /// 取引が削除された
    #[serde(rename_all = "camelCase")]
    TransactionDeleted {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        transaction_id: String,
    },

    /// 取引が更新された
    #[serde(rename_all = "camelCase")]
    TransactionUpdated {
        #[serde(flatten)]
        common: AccountEventCommonProps,
        #[serde(flatten)]
        props: TransactionProps,
        transaction_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_common_props() -> AccountEventCommonProps {
        AccountEventCommonProps {
            account_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            at: "2024-01-01T00:00:00Z".to_string(),
            id: "evt-123".to_string(),
            protocol_version: 3,
        }
    }

    fn create_transaction_props() -> TransactionProps {
        TransactionProps {
            amount: "1000".to_string(),
            category_id: "cat-123".to_string(),
            comment: "Test transaction".to_string(),
            date: "2024-01-01".to_string(),
        }
    }

    #[test]
    fn test_account_created_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::AccountCreated {
            common: create_common_props(),
            name: "Test Account".to_string(),
            owners: vec!["user-1".to_string(), "user-2".to_string()],
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "name": "Test Account",
            "owners": ["user-1", "user-2"],
            "protocolVersion": 3,
            "type": "accountCreated"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_account_deleted_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::AccountDeleted {
            common: create_common_props(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "protocolVersion": 3,
            "type": "accountDeleted"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_account_updated_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::AccountUpdated {
            common: create_common_props(),
            name: "Updated Account".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "name": "Updated Account",
            "protocolVersion": 3,
            "type": "accountUpdated"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_owner_added_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::OwnerAdded {
            common: create_common_props(),
            owner: "user-3".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "owner": "user-3",
            "protocolVersion": 3,
            "type": "ownerAdded"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_owner_removed_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::OwnerRemoved {
            common: create_common_props(),
            owner: "user-1".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "owner": "user-1",
            "protocolVersion": 3,
            "type": "ownerRemoved"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_category_added_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::CategoryAdded {
            category_id: "cat-456".to_string(),
            common: create_common_props(),
            name: "Food".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-456",
            "id": "evt-123",
            "name": "Food",
            "protocolVersion": 3,
            "type": "categoryAdded"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_category_updated_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::CategoryUpdated {
            category_id: "cat-456".to_string(),
            common: create_common_props(),
            name: "Groceries".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-456",
            "id": "evt-123",
            "name": "Groceries",
            "protocolVersion": 3,
            "type": "categoryUpdated"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_category_deleted_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::CategoryDeleted {
            category_id: "cat-456".to_string(),
            common: create_common_props(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-456",
            "id": "evt-123",
            "protocolVersion": 3,
            "type": "categoryDeleted"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_transaction_added_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::TransactionAdded {
            common: create_common_props(),
            props: create_transaction_props(),
            transaction_id: "txn-789".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "1000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-123",
            "comment": "Test transaction",
            "date": "2024-01-01",
            "id": "evt-123",
            "protocolVersion": 3,
            "transactionId": "txn-789",
            "type": "transactionAdded"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_transaction_updated_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::TransactionUpdated {
            common: create_common_props(),
            props: TransactionProps {
                amount: "2000".to_string(),
                category_id: "cat-456".to_string(),
                comment: "Updated transaction".to_string(),
                date: "2024-01-02".to_string(),
            },
            transaction_id: "txn-789".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "2000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-456",
            "comment": "Updated transaction",
            "date": "2024-01-02",
            "id": "evt-123",
            "protocolVersion": 3,
            "transactionId": "txn-789",
            "type": "transactionUpdated"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_transaction_deleted_serialization() -> anyhow::Result<()> {
        let event = AccountEvent::TransactionDeleted {
            common: create_common_props(),
            transaction_id: "txn-789".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let deserialized: AccountEvent = serde_json::from_str(&json)?;

        assert_eq!(event, deserialized);

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "protocolVersion": 3,
            "transactionId": "txn-789",
            "type": "transactionDeleted"
        });

        let json_value: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_flatten_common_props() -> anyhow::Result<()> {
        // Verify that common props are flattened into the event JSON
        let event = AccountEvent::AccountDeleted {
            common: create_common_props(),
        };

        let json = serde_json::to_string(&event)?;
        let json_value: serde_json::Value = serde_json::from_str(&json)?;

        // Common props should be at the top level, not nested under "common"
        assert!(json_value.get("common").is_none());

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "at": "2024-01-01T00:00:00Z",
            "id": "evt-123",
            "protocolVersion": 3,
            "type": "accountDeleted"
        });

        assert_eq!(json_value, expected);

        Ok(())
    }

    #[test]
    fn test_flatten_transaction_props() -> anyhow::Result<()> {
        // Verify that transaction props are flattened into the event JSON
        let event = AccountEvent::TransactionAdded {
            common: create_common_props(),
            props: create_transaction_props(),
            transaction_id: "txn-789".to_string(),
        };

        let json = serde_json::to_string(&event)?;
        let json_value: serde_json::Value = serde_json::from_str(&json)?;

        // Transaction props should be at the top level, not nested under "props"
        assert!(json_value.get("props").is_none());

        let expected = serde_json::json!({
            "accountId": "550e8400-e29b-41d4-a716-446655440000",
            "amount": "1000",
            "at": "2024-01-01T00:00:00Z",
            "categoryId": "cat-123",
            "comment": "Test transaction",
            "date": "2024-01-01",
            "id": "evt-123",
            "protocolVersion": 3,
            "transactionId": "txn-789",
            "type": "transactionAdded"
        });

        assert_eq!(json_value, expected);

        Ok(())
    }
}
