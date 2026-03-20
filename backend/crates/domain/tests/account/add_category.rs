use crate::helper;
use domain::Account;
use domain::AccountCommand;
use domain::AccountError;
use domain::AccountEvent;
use domain::CategoryId;

#[test]
fn test_add_category() -> anyhow::Result<()> {
    let (mut account, _, _) = helper::create_active_account()?;
    let category_id = CategoryId::generate();

    let command = AccountCommand::AddCategory {
        category_id,
        name: "Food".to_string(),
    };
    let events = account.handle_command(command)?;
    assert_eq!(events.len(), 1);

    match &events[0] {
        AccountEvent::CategoryAdded {
            category_id: cid,
            name,
            ..
        } => {
            assert_eq!(cid, &category_id.to_string());
            assert_eq!(name, "Food");
        }
        event => anyhow::bail!("Expected CategoryAdded event, got {:?}", event),
    }

    helper::apply_events(&mut account, &events);
    match &account {
        Account::Active { categories, .. } => {
            let category = categories
                .get(&category_id)
                .ok_or_else(|| anyhow::anyhow!("Category not found in state"))?;
            assert_eq!(category.name, "Food");
            assert!(!category.deleted);
            Ok(())
        }
        Account::Empty => anyhow::bail!("Expected Active account, got Empty"),
    }
}

#[test]
fn test_add_category_empty_name() -> anyhow::Result<()> {
    let (account, _, _) = helper::create_active_account()?;
    let category_id = CategoryId::generate();

    let command = AccountCommand::AddCategory {
        category_id,
        name: "  ".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::EmptyCategoryName) => Ok(()),
        Ok(_) => anyhow::bail!("Expected EmptyCategoryName error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected EmptyCategoryName error, got {:?}", e),
    }
}

#[test]
fn test_add_category_not_found() -> anyhow::Result<()> {
    let account = Account::new();
    let category_id = CategoryId::generate();

    let command = AccountCommand::AddCategory {
        category_id,
        name: "Food".to_string(),
    };

    match account.handle_command(command) {
        Err(AccountError::AccountNotFound) => Ok(()),
        Ok(_) => anyhow::bail!("Expected AccountNotFound error, but command succeeded"),
        Err(e) => anyhow::bail!("Expected AccountNotFound error, got {:?}", e),
    }
}
