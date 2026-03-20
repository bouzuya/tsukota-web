use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::AccountEventCommonProps;
use domain::AccountId;
use domain::UserId;

#[test]
fn test_create_account() -> anyhow::Result<()> {
    let account = Account::new();
    let account_id = AccountId::generate();
    let user_id = UserId::generate();
    let command = AccountCommand::CreateAccount {
        account_id,
        name: "My Account".to_string(),
        owners: vec![user_id],
    };

    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::AccountCreated { name, owners, .. } => {
            assert_eq!(name, "My Account");
            assert_eq!(owners, &vec![user_id.to_string()]);
            Ok(())
        }
        event => anyhow::bail!("Expected AccountCreated event, got {:?}", event),
    }
}

#[test]
fn test_create_account_empty_name() -> anyhow::Result<()> {
    let account = Account::new();
    let account_id = AccountId::generate();
    let user_id = UserId::generate();
    let command = AccountCommand::CreateAccount {
        account_id,
        name: "  ".to_string(),
        owners: vec![user_id],
    };

    match account.handle_command(command) {
        Err(AccountError::EmptyAccountName) => Ok(()),
        Ok(_) => anyhow::bail!("Expected EmptyAccountName error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected EmptyAccountName error, got {:?}", e),
    }
}

#[test]
fn test_create_account_already_exists() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let account_id = AccountId::generate();
    let user_id = UserId::generate();
    let command = AccountCommand::CreateAccount {
        account_id,
        name: "Another Account".to_string(),
        owners: vec![user_id],
    };

    match account.handle_command(command) {
        Err(AccountError::AccountAlreadyExists) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountAlreadyExists error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountAlreadyExists error, got {:?}", e),
    }
}

#[test]
fn test_account_from_events() -> anyhow::Result<()> {
    let account_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let user_uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    let common = AccountEventCommonProps {
        account_id: account_uuid.to_string(),
        at: "2024-01-01T00:00:00Z".to_string(),
        id: "evt-1".to_string(),
        protocol_version: Account::PROTOCOL_VERSION,
    };

    let events = vec![AccountEvent::AccountCreated {
        name: "My Account".to_string(),
        owners: vec![user_uuid.to_string()],
        common,
    }];

    let account = Account::from_events(events);
    match account {
        Account::Active {
            id, name, owners, ..
        } => {
            assert_eq!(id.to_string(), account_uuid);
            assert_eq!(name, "My Account");
            let expected_user_id: UserId = user_uuid.parse()?;
            assert!(owners.contains(&expected_user_id));
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}
