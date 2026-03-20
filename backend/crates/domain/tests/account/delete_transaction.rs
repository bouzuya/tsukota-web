use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::TransactionId;

#[test]
fn test_delete_transaction() -> anyhow::Result<()> {
    let (mut account, _, _, _, transaction_id) = helper::create_account_with_transaction()?;

    let command = AccountCommand::DeleteTransaction { transaction_id };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::TransactionDeleted {
            transaction_id: tid,
            ..
        } => {
            assert_eq!(tid, &transaction_id.to_string());
        }
        event => anyhow::bail!("Expected TransactionDeleted event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { transactions, .. } => {
            assert!(!transactions.contains_key(&transaction_id));
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_delete_transaction_not_found() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let unknown_transaction_id = TransactionId::generate();

    let command = AccountCommand::DeleteTransaction {
        transaction_id: unknown_transaction_id,
    };

    match account.handle_command(command) {
        Err(AccountError::TransactionNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected TransactionNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected TransactionNotFound error, got {:?}", e),
    }
}

#[test]
fn test_delete_transaction_account_not_found() -> anyhow::Result<()> {
    let account = Account::new();
    let transaction_id = TransactionId::generate();

    let command = AccountCommand::DeleteTransaction { transaction_id };

    match account.handle_command(command) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
