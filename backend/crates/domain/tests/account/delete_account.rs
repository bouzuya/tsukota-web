use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;

#[test]
fn test_delete_account() -> anyhow::Result<()> {
    let (mut account, _, _) = helper::create_active_account()?;

    let events = account.handle_command(AccountCommand::DeleteAccount)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::AccountDeleted { .. } => {}
        event => anyhow::bail!("Expected AccountDeleted event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match account {
        Account::Empty => Ok(()),
        Account::Active { .. } => anyhow::bail!("Expected Empty account after deletion"),
    }
}

#[test]
fn test_delete_account_not_found() -> anyhow::Result<()> {
    let account = Account::new();

    match account.handle_command(AccountCommand::DeleteAccount) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
