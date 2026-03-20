use domain::Account;
use domain::AccountCommand;
use domain::AccountEvent;
use domain::AccountId;
use domain::CategoryId;
use domain::TransactionId;
use domain::UserId;

/// アカウントを作成し、イベント適用済みの状態を返す
pub fn create_active_account() -> anyhow::Result<(Account, AccountId, UserId)> {
    let mut account = Account::new();
    let account_id = AccountId::generate();
    let user_id = UserId::generate();
    let command = AccountCommand::CreateAccount {
        account_id,
        name: "Test Account".to_string(),
        owners: vec![user_id],
    };
    let events = account.handle_command(command)?;
    apply_events(&mut account, &events);
    Ok((account, account_id, user_id))
}

/// アカウントに区分を追加した状態を返す
pub fn create_account_with_category() -> anyhow::Result<(Account, AccountId, UserId, CategoryId)> {
    let (mut account, account_id, user_id) = create_active_account()?;
    let category_id = CategoryId::generate();
    let command = AccountCommand::AddCategory {
        category_id,
        name: "Food".to_string(),
    };
    let events = account.handle_command(command)?;
    apply_events(&mut account, &events);
    Ok((account, account_id, user_id, category_id))
}

/// アカウントに区分と取引を追加した状態を返す
pub fn create_account_with_transaction()
-> anyhow::Result<(Account, AccountId, UserId, CategoryId, TransactionId)> {
    let (mut account, account_id, user_id, category_id) = create_account_with_category()?;
    let transaction_id = TransactionId::generate();
    let command = AccountCommand::AddTransaction {
        transaction_id,
        amount: "1000".to_string(),
        category_id,
        comment: "Test".to_string(),
        date: "2024-01-01".to_string(),
    };
    let events = account.handle_command(command)?;
    apply_events(&mut account, &events);
    Ok((account, account_id, user_id, category_id, transaction_id))
}

/// イベントを適用するヘルパー
pub fn apply_events(account: &mut Account, events: &[AccountEvent]) {
    for event in events {
        account.apply_event(event);
    }
}
