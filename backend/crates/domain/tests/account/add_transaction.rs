use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::CategoryId;
use domain::TransactionId;

#[test]
fn test_add_transaction() -> anyhow::Result<()> {
    let (mut account, _, _, category_id) = helper::create_account_with_category()?;
    let transaction_id = TransactionId::generate();

    let command = AccountCommand::AddTransaction {
        transaction_id,
        amount: "1500".to_string(),
        category_id,
        comment: "Lunch".to_string(),
        date: "2024-03-15".to_string(),
    };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::TransactionAdded {
            transaction_id: tid,
            props,
            ..
        } => {
            assert_eq!(tid, &transaction_id.to_string());
            assert_eq!(props.amount, "1500");
            assert_eq!(props.category_id, category_id.to_string());
            assert_eq!(props.comment, "Lunch");
            assert_eq!(props.date, "2024-03-15");
        }
        event => anyhow::bail!("Expected TransactionAdded event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { transactions, .. } => {
            let transaction = transactions
                .get(&transaction_id)
                .ok_or_else(|| anyhow::anyhow!("Transaction not found in state"))?;
            assert_eq!(transaction.amount, "1500");
            assert_eq!(transaction.category_id, category_id);
            assert_eq!(transaction.comment, "Lunch");
            assert_eq!(transaction.date, "2024-03-15");
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_add_transaction_empty_amount() -> anyhow::Result<()> {
    let (account, _, _, category_id) = helper::create_account_with_category()?;
    let transaction_id = TransactionId::generate();

    let command = AccountCommand::AddTransaction {
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
fn test_add_transaction_category_not_found() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let transaction_id = TransactionId::generate();
    let unknown_category_id = CategoryId::generate();

    let command = AccountCommand::AddTransaction {
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
fn test_add_transaction_invalid_date() -> anyhow::Result<()> {
    let (account, _, _, category_id) = helper::create_account_with_category()?;
    let transaction_id = TransactionId::generate();

    let command = AccountCommand::AddTransaction {
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

#[test]
fn test_add_transaction_deleted_category() -> anyhow::Result<()> {
    // 削除済み区分でも取引を追加できることを確認（仕様）
    let (mut account, _, _, category_id) = helper::create_account_with_category()?;

    // 区分を削除
    let delete_events = account.handle_command(AccountCommand::DeleteCategory { category_id })?;
    helper::apply_events(&mut account, &delete_events);

    // 削除済み区分で取引を追加
    let transaction_id = TransactionId::generate();
    let command = AccountCommand::AddTransaction {
        transaction_id,
        amount: "500".to_string(),
        category_id,
        comment: "Test".to_string(),
        date: "2024-01-01".to_string(),
    };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::TransactionAdded { .. } => Ok(()),
        event => anyhow::bail!("Expected TransactionAdded event, got {:?}", event),
    }
}
