use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::CategoryId;
use domain::TransactionId;

#[test]
fn test_update_transaction() -> anyhow::Result<()> {
    let (mut account, _, _, category_id, transaction_id) =
        helper::create_account_with_transaction()?;

    let command = AccountCommand::UpdateTransaction {
        transaction_id,
        amount: "2000".to_string(),
        category_id,
        comment: "Dinner".to_string(),
        date: "2024-03-20".to_string(),
    };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::TransactionUpdated {
            transaction_id: tid,
            props,
            ..
        } => {
            assert_eq!(tid, &transaction_id.to_string());
            assert_eq!(props.amount, "2000");
            assert_eq!(props.comment, "Dinner");
            assert_eq!(props.date, "2024-03-20");
        }
        event => anyhow::bail!("Expected TransactionUpdated event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { transactions, .. } => {
            let transaction = transactions
                .get(&transaction_id)
                .ok_or_else(|| anyhow::anyhow!("Transaction not found in state"))?;
            assert_eq!(transaction.amount, "2000");
            assert_eq!(transaction.comment, "Dinner");
            assert_eq!(transaction.date, "2024-03-20");
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_update_transaction_not_found() -> anyhow::Result<()> {
    let (account, _, _, category_id) = helper::create_account_with_category()?;
    let unknown_transaction_id = TransactionId::generate();

    let command = AccountCommand::UpdateTransaction {
        transaction_id: unknown_transaction_id,
        amount: "1000".to_string(),
        category_id,
        comment: "Test".to_string(),
        date: "2024-01-01".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::TransactionNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected TransactionNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected TransactionNotFound error, got {:?}", e),
    }
}

#[test]
fn test_update_transaction_empty_amount() -> anyhow::Result<()> {
    let (account, _, _, category_id, transaction_id) = helper::create_account_with_transaction()?;

    let command = AccountCommand::UpdateTransaction {
        transaction_id,
        amount: "  ".to_string(),
        category_id,
        comment: "Test".to_string(),
        date: "2024-01-01".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::EmptyAmount) => Ok(()),
        Ok(_) => anyhow::bail!("Expected EmptyAmount error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected EmptyAmount error, got {:?}", e),
    }
}

#[test]
fn test_update_transaction_category_not_found() -> anyhow::Result<()> {
    let (account, _, _, _, transaction_id) = helper::create_account_with_transaction()?;
    let unknown_category_id = CategoryId::generate();

    let command = AccountCommand::UpdateTransaction {
        transaction_id,
        amount: "1000".to_string(),
        category_id: unknown_category_id,
        comment: "Test".to_string(),
        date: "2024-01-01".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::CategoryNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected CategoryNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected CategoryNotFound error, got {:?}", e),
    }
}

#[test]
fn test_update_transaction_invalid_date() -> anyhow::Result<()> {
    let (account, _, _, category_id, transaction_id) = helper::create_account_with_transaction()?;

    let command = AccountCommand::UpdateTransaction {
        transaction_id,
        amount: "1000".to_string(),
        category_id,
        comment: "Test".to_string(),
        date: "20240101".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::InvalidDateFormat) => Ok(()),
        Ok(_) => anyhow::bail!("Expected InvalidDateFormat error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected InvalidDateFormat error, got {:?}", e),
    }
}
