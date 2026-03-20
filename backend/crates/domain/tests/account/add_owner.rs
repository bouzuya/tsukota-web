use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::UserId;

#[test]
fn test_add_owner() -> anyhow::Result<()> {
    let (mut account, _, _) = helper::create_active_account()?;
    let new_owner = UserId::generate();

    let command = AccountCommand::AddOwner { owner: new_owner };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::OwnerAdded { owner, .. } => {
            assert_eq!(owner, &new_owner.to_string());
        }
        event => anyhow::bail!("Expected OwnerAdded event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { owners, .. } => {
            assert!(owners.contains(&new_owner));
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_add_owner_already_exists() -> anyhow::Result<()> {
    let (account, _, user_id) = helper::create_active_account()?;

    let command = AccountCommand::AddOwner { owner: user_id };

    match account.handle_command(command) {
        Err(AccountError::OwnerAlreadyExists) => Ok(()),
        Ok(_) => anyhow::bail!("Expected OwnerAlreadyExists error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected OwnerAlreadyExists error, got {:?}", e),
    }
}

#[test]
fn test_add_owner_not_found() -> anyhow::Result<()> {
    let account = Account::new();
    let owner = UserId::generate();

    let command = AccountCommand::AddOwner { owner };

    match account.handle_command(command) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
