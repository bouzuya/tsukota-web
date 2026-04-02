use crate::FirestoreUserRepository;
use crate::repository::Repository;
use crate::schema::AccountEventStreamDocumentData;
use crate::schema::QueryAccountDocumentData;
use application::error::ApplicationError;
use application::repository::AccountRepository;
use application::repository::UserRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use domain::AccountEvent;
use domain::AccountId;
use domain::User;
use domain::UserCommand;
use domain::UserId;

/// Firestore-based event store implementation
#[derive(Clone)]
pub struct FirestoreAccountRepository {
    firestore: Firestore,
    user_repository: FirestoreUserRepository,
}

impl FirestoreAccountRepository {
    /// Create a new FirestoreAccountRepository with the given firestore instance
    pub fn new(firestore: Firestore) -> Self {
        let user_repository = FirestoreUserRepository::new(firestore.clone());
        Self {
            firestore,
            user_repository,
        }
    }

    /// Get the path to a query account document: `accounts/{account_id}`
    fn query_account_document_path(account_id: &AccountId) -> String {
        format!("accounts/{}", account_id)
    }

    /// Get the path to a query event document: `accounts/{account_id}/events/{event_id}`
    fn query_event_document_path(account_id: &AccountId, event_id: &str) -> String {
        format!("accounts/{}/events/{}", account_id, event_id)
    }
}

impl Repository for FirestoreAccountRepository {
    type Event = AccountEvent;
    type EventAt = String;
    type EventId = String;
    type EventStream = AccountEventStreamDocumentData;
    type EventStreamId = AccountId;

    fn aggregate_name() -> String {
        "account".to_string()
    }

    fn firestore(&self) -> &Firestore {
        &self.firestore
    }

    fn get_event_at(event: &Self::Event) -> Self::EventAt {
        match event {
            AccountEvent::AccountCreated { common, .. }
            | AccountEvent::AccountDeleted { common, .. }
            | AccountEvent::AccountUpdated { common, .. }
            | AccountEvent::CategoryAdded { common, .. }
            | AccountEvent::CategoryDeleted { common, .. }
            | AccountEvent::CategoryUpdated { common, .. }
            | AccountEvent::OwnerAdded { common, .. }
            | AccountEvent::OwnerRemoved { common, .. }
            | AccountEvent::TransactionAdded { common, .. }
            | AccountEvent::TransactionDeleted { common, .. }
            | AccountEvent::TransactionUpdated { common, .. } => common.at.clone(),
        }
    }

    fn get_event_id(event: &Self::Event) -> Self::EventId {
        match event {
            AccountEvent::AccountCreated { common, .. }
            | AccountEvent::AccountDeleted { common, .. }
            | AccountEvent::AccountUpdated { common, .. }
            | AccountEvent::CategoryAdded { common, .. }
            | AccountEvent::CategoryDeleted { common, .. }
            | AccountEvent::CategoryUpdated { common, .. }
            | AccountEvent::OwnerAdded { common, .. }
            | AccountEvent::OwnerRemoved { common, .. }
            | AccountEvent::TransactionAdded { common, .. }
            | AccountEvent::TransactionDeleted { common, .. }
            | AccountEvent::TransactionUpdated { common, .. } => common.id.clone(),
        }
    }

    fn new_event_stream(
        event_stream_id: &Self::EventStreamId,
        events: &[Self::Event],
        stored_event_stream: Option<Self::EventStream>,
    ) -> Self::EventStream {
        let mut owners = match &stored_event_stream {
            Some(stored) => stored.owners.clone(),
            None => match &events[0] {
                AccountEvent::AccountCreated { owners, .. } => owners.clone(),
                AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::OwnerAdded { .. }
                | AccountEvent::OwnerRemoved { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => vec![],
            },
        };

        // イベントからオーナー変更を適用
        for event in events {
            match event {
                AccountEvent::OwnerAdded { owner, .. } => {
                    if !owners.contains(owner) {
                        owners.push(owner.clone());
                    }
                }
                AccountEvent::OwnerRemoved { owner, .. } => {
                    owners.retain(|o| o != owner);
                }
                AccountEvent::AccountCreated { .. }
                | AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => {}
            }
        }

        let last_event = events.last().expect("events is non-empty");
        AccountEventStreamDocumentData {
            id: event_stream_id.to_string(),
            last_event_id: Self::get_event_id(last_event),
            owners,
            protocol_version: domain::Account::PROTOCOL_VERSION,
            updated_at: Self::get_event_at(last_event),
        }
    }
}

#[async_trait]
impl AccountRepository for FirestoreAccountRepository {
    async fn load_events(
        &self,
        account_id: &AccountId,
    ) -> Result<Vec<AccountEvent>, ApplicationError> {
        Repository::load_events(self, account_id)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }

    async fn save_events(
        &self,
        account_id: &AccountId,
        events: Vec<AccountEvent>,
    ) -> Result<(), ApplicationError> {
        // オーナー変更を収集 (User 集約の更新に使用)
        let user_updates = Self::collect_user_updates(&events);

        let account_id_owned = *account_id;
        let events_clone = events.clone();
        let firestore = self.firestore.clone();
        Repository::save_events(
            self,
            *account_id,
            events,
            Box::new(move |transaction| {
                Box::pin(async move {
                    // accounts/* (クエリ用コレクション) への書き込み
                    Self::build_query_account_writes_in_tx(
                        &firestore,
                        &account_id_owned,
                        &events_clone,
                        transaction,
                    )
                    .await?;
                    Ok(())
                })
            }),
        )
        .await
        .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        // User 集約の更新 (別トランザクション)
        self.update_user_aggregates(account_id, user_updates)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;

        Ok(())
    }
}

impl FirestoreAccountRepository {
    /// AccountEvent からオーナー変更を収集する
    fn collect_user_updates(events: &[AccountEvent]) -> Vec<(UserId, UserUpdateAction)> {
        let mut updates = Vec::new();

        for event in events {
            match event {
                AccountEvent::AccountCreated { owners, .. } => {
                    for owner in owners {
                        if let Ok(user_id) = owner.parse::<UserId>() {
                            updates.push((user_id, UserUpdateAction::AddAccount));
                        }
                    }
                }
                AccountEvent::OwnerAdded { owner, .. } => {
                    if let Ok(user_id) = owner.parse::<UserId>() {
                        updates.push((user_id, UserUpdateAction::AddAccount));
                    }
                }
                AccountEvent::OwnerRemoved { owner, .. } => {
                    if let Ok(user_id) = owner.parse::<UserId>() {
                        updates.push((user_id, UserUpdateAction::RemoveAccount));
                    }
                }
                AccountEvent::AccountDeleted { .. }
                | AccountEvent::AccountUpdated { .. }
                | AccountEvent::CategoryAdded { .. }
                | AccountEvent::CategoryDeleted { .. }
                | AccountEvent::CategoryUpdated { .. }
                | AccountEvent::TransactionAdded { .. }
                | AccountEvent::TransactionDeleted { .. }
                | AccountEvent::TransactionUpdated { .. } => {}
            }
        }

        updates
    }

    /// User 集約を更新する (別トランザクション)
    async fn update_user_aggregates(
        &self,
        account_id: &AccountId,
        user_updates: Vec<(UserId, UserUpdateAction)>,
    ) -> Result<(), ApplicationError> {
        for (user_id, action) in user_updates {
            // User 集約を読み込み
            let user_events = UserRepository::load_events(&self.user_repository, &user_id).await?;
            let user = User::from_events(user_events);

            // コマンドを実行
            let command = match action {
                UserUpdateAction::AddAccount => UserCommand::AddAccount {
                    user_id,
                    account_id: *account_id,
                },
                UserUpdateAction::RemoveAccount => UserCommand::RemoveAccount {
                    user_id,
                    account_id: *account_id,
                },
            };

            let new_events = match user.handle_command(command) {
                Ok(events) => events,
                Err(domain::UserError::AccountAlreadyAdded) => {
                    // 既に追加済み - スキップ
                    continue;
                }
                Err(domain::UserError::AccountNotFound) => {
                    // 既に削除済み - スキップ
                    continue;
                }
                Err(e) => {
                    return Err(ApplicationError::Repository(e.to_string()));
                }
            };

            // 新しいイベントを保存
            if !new_events.is_empty() {
                UserRepository::save_events(&self.user_repository, &user_id, new_events).await?;
            }
        }

        Ok(())
    }

    /// `accounts/*` への書き込みをトランザクション内で実行する
    async fn build_query_account_writes_in_tx(
        firestore: &Firestore,
        account_id: &AccountId,
        events: &[AccountEvent],
        transaction: &mut bouzuya_firestore_client::Transaction,
    ) -> Result<(), bouzuya_firestore_client::Error> {
        // accounts/{account_id}/events/{event_id} へのイベント書き込み
        for event in events {
            let event_id = Self::get_event_id(event);
            let document_path = Self::query_event_document_path(account_id, &event_id);
            let document_ref = firestore.doc(document_path)?;
            transaction.create(&document_ref, event)?;
        }

        // accounts/{account_id} へのアカウントドキュメント書き込み
        let document_path = Self::query_account_document_path(account_id);
        let document_ref = firestore.doc(document_path)?;
        let document_snapshot = transaction.get(&document_ref).await?;

        match document_snapshot.data::<QueryAccountDocumentData>() {
            None => {
                // 新規作成 (AccountCreated イベントがある場合のみ)
                if let Some(AccountEvent::AccountCreated { name, owners, .. }) = events.first() {
                    let document_data = QueryAccountDocumentData {
                        deleted_at: None,
                        id: account_id.to_string(),
                        name: name.clone(),
                        owners: owners.clone(),
                    };
                    transaction.create(&document_ref, &document_data)?;
                }
            }
            Some(result) => {
                let mut document_data: QueryAccountDocumentData = result?;

                for event in events {
                    match event {
                        AccountEvent::AccountUpdated { name, .. } => {
                            document_data.name = name.clone();
                        }
                        AccountEvent::AccountDeleted { common, .. } => {
                            document_data.deleted_at = Some(common.at.clone());
                        }
                        AccountEvent::OwnerAdded { owner, .. } => {
                            if !document_data.owners.contains(owner) {
                                document_data.owners.push(owner.clone());
                            }
                        }
                        AccountEvent::OwnerRemoved { owner, .. } => {
                            document_data.owners.retain(|o| o != owner);
                        }
                        AccountEvent::AccountCreated { .. }
                        | AccountEvent::CategoryAdded { .. }
                        | AccountEvent::CategoryDeleted { .. }
                        | AccountEvent::CategoryUpdated { .. }
                        | AccountEvent::TransactionAdded { .. }
                        | AccountEvent::TransactionDeleted { .. }
                        | AccountEvent::TransactionUpdated { .. } => {}
                    }
                }

                transaction.update(
                    &document_ref,
                    &document_data,
                    Precondition {
                        exists: Some(true),
                        last_update_time: None,
                    },
                )?;
            }
        }

        Ok(())
    }
}

/// Action to perform on a user document
#[derive(Clone, Debug)]
enum UserUpdateAction {
    AddAccount,
    RemoveAccount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use application::repository::AccountRepository;
    use bouzuya_firestore_client::FirestoreOptions;
    use domain::AccountEventCommonProps;

    /// テスト用のリポジトリを生成する
    async fn setup_repository() -> anyhow::Result<FirestoreAccountRepository> {
        let firestore = Firestore::new(FirestoreOptions {
            project_id: Some("demo-project".to_string()),
        })?;
        Ok(FirestoreAccountRepository::new(firestore))
    }

    /// テスト用の AccountCreated イベントを生成する
    fn account_created_event(account_id: &AccountId, event_id: &str, at: &str) -> AccountEvent {
        AccountEvent::AccountCreated {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: at.to_string(),
                id: event_id.to_string(),
                protocol_version: 3,
            },
            name: "テストアカウント".to_string(),
            // オーナーを空にしてユーザー集約の更新を回避する
            owners: vec![],
        }
    }

    #[tokio::test]
    async fn test_load_events_empty() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();

        let events = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(events, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_events_single_event() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let event = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");

        AccountRepository::save_events(&repo, &account_id, vec![event.clone()]).await?;
        let loaded = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(loaded, vec![event]);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_events_sorted_by_at() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let account_id = AccountId::generate();
        let event1 = account_created_event(&account_id, "evt-001", "2024-01-01T00:00:00Z");
        let event2 = AccountEvent::AccountUpdated {
            common: AccountEventCommonProps {
                account_id: account_id.to_string(),
                at: "2024-01-02T00:00:00Z".to_string(),
                id: "evt-002".to_string(),
                protocol_version: 3,
            },
            name: "更新後アカウント".to_string(),
        };

        AccountRepository::save_events(&repo, &account_id, vec![event1.clone()]).await?;
        AccountRepository::save_events(&repo, &account_id, vec![event2.clone()]).await?;

        let loaded = AccountRepository::load_events(&repo, &account_id).await?;

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], event1);
        assert_eq!(loaded[1], event2);
        Ok(())
    }
}
