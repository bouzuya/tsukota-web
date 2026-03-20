use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::CategoryId;

#[test]
fn test_delete_category() -> anyhow::Result<()> {
    let (mut account, _, _, category_id) = helper::create_account_with_category()?;

    let command = AccountCommand::DeleteCategory { category_id };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::CategoryDeleted {
            category_id: cid, ..
        } => {
            assert_eq!(cid, &category_id.to_string());
        }
        event => anyhow::bail!("Expected CategoryDeleted event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { categories, .. } => {
            let category = categories
                .get(&category_id)
                .ok_or_else(|| anyhow::anyhow!("Category not found in state"))?;
            assert!(category.deleted);
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_delete_category_not_found() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let unknown_category_id = CategoryId::generate();

    let command = AccountCommand::DeleteCategory {
        category_id: unknown_category_id,
    };

    match account.handle_command(command) {
        Err(AccountError::CategoryNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected CategoryNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected CategoryNotFound error, got {:?}", e),
    }
}

#[test]
fn test_delete_category_already_deleted() -> anyhow::Result<()> {
    let (mut account, _, _, category_id) = helper::create_account_with_category()?;

    // 区分を削除
    let delete_events = account.handle_command(AccountCommand::DeleteCategory { category_id })?;
    helper::apply_events(&mut account, &delete_events);

    // 再度削除を試みる
    let command = AccountCommand::DeleteCategory { category_id };

    match account.handle_command(command) {
        Err(AccountError::CategoryAlreadyDeleted) => Ok(()),
        Ok(_) => anyhow::bail!("Expected CategoryAlreadyDeleted error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected CategoryAlreadyDeleted error, got {:?}", e),
    }
}
