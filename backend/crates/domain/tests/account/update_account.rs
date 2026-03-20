use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;

#[test]
fn test_update_account() -> anyhow::Result<()> {
    let (mut account, _, _) = helper::create_active_account()?;

    let command = AccountCommand::UpdateAccount {
        name: "Updated Name".to_string(),
    };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::AccountUpdated { name, .. } => {
            assert_eq!(name, "Updated Name");
        }
        event => anyhow::bail!("Expected AccountUpdated event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { name, .. } => {
            assert_eq!(name, "Updated Name");
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_update_account_empty_name() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;

    let command = AccountCommand::UpdateAccount {
        name: "  ".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::EmptyAccountName) => Ok(()),
        Ok(_) => anyhow::bail!("Expected EmptyAccountName error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected EmptyAccountName error, got {:?}", e),
    }
}

#[test]
fn test_update_account_not_found() -> anyhow::Result<()> {
    let account = Account::new();

    let command = AccountCommand::UpdateAccount {
        name: "Name".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
