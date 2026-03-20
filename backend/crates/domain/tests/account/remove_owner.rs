use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::UserId;

#[test]
fn test_remove_owner() -> anyhow::Result<()> {
    // 2人のオーナーがいる状態を作る
    let (mut account, _, user_id) = helper::create_active_account()?;
    let second_owner = UserId::generate();
    let add_events = account.handle_command(AccountCommand::AddOwner {
        owner: second_owner,
    })?;
    helper::apply_events(&mut account, &add_events);

    // 最初のオーナーを削除
    let command = AccountCommand::RemoveOwner { owner: user_id };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::OwnerRemoved { owner, .. } => {
            assert_eq!(owner, &user_id.to_string());
        }
        event => anyhow::bail!("Expected OwnerRemoved event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { owners, .. } => {
            assert!(!owners.contains(&user_id));
            assert!(owners.contains(&second_owner));
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_remove_owner_not_found() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let unknown_owner = UserId::generate();

    let command = AccountCommand::RemoveOwner {
        owner: unknown_owner,
    };

    match account.handle_command(command) {
        Err(AccountError::OwnerNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected OwnerNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected OwnerNotFound error, got {:?}", e),
    }
}

#[test]
fn test_remove_owner_last_owner() -> anyhow::Result<()> {
    let (account, _, user_id) = helper::create_active_account()?;

    let command = AccountCommand::RemoveOwner { owner: user_id };

    match account.handle_command(command) {
        Err(AccountError::CannotRemoveLastOwner) => Ok(()),
        Ok(_) => anyhow::bail!("Expected CannotRemoveLastOwner error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected CannotRemoveLastOwner error, got {:?}", e),
    }
}

#[test]
fn test_remove_owner_account_not_found() -> anyhow::Result<()> {
    let account = Account::new();
    let owner = UserId::generate();

    let command = AccountCommand::RemoveOwner { owner };

    match account.handle_command(command) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
