use std::fmt::Display;

use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::Precondition;
use bouzuya_firestore_client::Transaction as FirestoreTransaction;
use bouzuya_firestore_client::TransactionOptions;
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
type SaveEventsCallback = Box<
    dyn for<'a> FnOnce(
            &'a mut FirestoreTransaction,
        ) -> BoxFuture<'a, Result<(), bouzuya_firestore_client::Error>>
        + Send
        + Sync,
>;

/// リポジトリ操作の共通エラー型
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub(crate) struct RepositoryError(Box<dyn std::error::Error + Send + Sync>);

/// Firestore ベースのイベントソーシングリポジトリの共通 trait
pub(crate) trait Repository {
    type Event: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static;
    type EventAt: Ord;
    type EventId: Display;
    type EventStream: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + 'static;
    type EventStreamId: Clone + Display + Send + Sync + 'static;

    /// イベントコレクションのパスを返す: `aggregates/{aggregate}/event_streams/{id}/events`
    fn event_collection_path(event_stream_id: &Self::EventStreamId) -> String {
        format!(
            "{}/events",
            Self::event_stream_document_path(event_stream_id)
        )
    }

    /// イベントドキュメントのパスを返す: `aggregates/{aggregate}/event_streams/{id}/events/{event_id}`
    fn event_document_path(
        event_stream_id: &Self::EventStreamId,
        event_id: &Self::EventId,
    ) -> String {
        format!(
            "{}/{}",
            Self::event_collection_path(event_stream_id),
            event_id
        )
    }

    /// 集約名を返す
    fn aggregate_name() -> String;

    /// イベントストリームコレクションのパスを返す: `aggregates/{aggregate}/event_streams`
    fn event_stream_collection_path() -> String {
        format!("aggregates/{}/event_streams", Self::aggregate_name())
    }

    /// イベントストリームドキュメントのパスを返す: `aggregates/{aggregate}/event_streams/{id}`
    fn event_stream_document_path(event_stream_id: &Self::EventStreamId) -> String {
        format!(
            "{}/{}",
            Self::event_stream_collection_path(),
            event_stream_id
        )
    }

    /// Firestore インスタンスへの参照を返す
    fn firestore(&self) -> &Firestore;

    /// イベントからタイムスタンプを取得する
    fn get_event_at(event: &Self::Event) -> Self::EventAt;

    /// イベントから ID を取得する
    fn get_event_id(event: &Self::Event) -> Self::EventId;

    /// イベントストリームからイベントを読み込む
    async fn load_events(
        &self,
        event_stream_id: &Self::EventStreamId,
    ) -> Result<Vec<Self::Event>, RepositoryError> {
        let collection_path = Self::event_collection_path(event_stream_id);
        let collection_ref = self
            .firestore()
            .collection(collection_path)
            .map_err(|e| RepositoryError(Box::new(e)))?;
        let document_refs = collection_ref
            .list_documents()
            .await
            .map_err(|e| RepositoryError(Box::new(e)))?;
        let document_snapshots = self
            .firestore()
            .get_all(document_refs)
            .await
            .map_err(|e| RepositoryError(Box::new(e)))?;
        let mut events = document_snapshots
            .into_iter()
            .map(|document_snapshot| {
                document_snapshot
                    .data::<Self::Event>()
                    .ok_or_else(|| RepositoryError(Box::from("event not found")))?
                    .map_err(|e| RepositoryError(Box::new(e)))
            })
            .collect::<Result<Vec<Self::Event>, RepositoryError>>()?;
        events.sort_by(|a, b| Self::get_event_at(a).cmp(&Self::get_event_at(b)));
        Ok(events)
    }

    /// イベントストリームドキュメントを生成する
    ///
    /// `stored_event_stream` が `Some` の場合は既存のイベントストリームの更新、
    /// `None` の場合は新規作成として扱う。
    fn new_event_stream(
        event_stream_id: &Self::EventStreamId,
        events: &[Self::Event],
        stored_event_stream: Option<Self::EventStream>,
    ) -> Self::EventStream;

    /// イベントをイベントストリームに保存する
    async fn save_events(
        &self,
        event_stream_id: Self::EventStreamId,
        events: Vec<Self::Event>,
        callback: SaveEventsCallback,
    ) -> Result<(), RepositoryError> {
        if events.is_empty() {
            return Ok(());
        }

        let firestore = self.firestore().clone();
        self.firestore()
            .run_transaction(
                |transaction| {
                    Box::pin(async move {
                        {
                            // イベントストリームドキュメントの読み込み (排他制御のために get を使用)
                            let document_path = Self::event_stream_document_path(&event_stream_id);
                            let document_ref = firestore.doc(document_path)?;
                            let document_snapshot = transaction.get(&document_ref).await?;

                            // イベントストリームドキュメントの書き込み
                            let stored_event_stream =
                                document_snapshot.data::<Self::EventStream>().transpose()?;
                            let is_update = stored_event_stream.is_some();
                            let event_stream = Self::new_event_stream(
                                &event_stream_id,
                                &events,
                                stored_event_stream,
                            );
                            if is_update {
                                // TODO: 排他制御
                                transaction.update(
                                    &document_ref,
                                    &event_stream,
                                    Precondition {
                                        exists: Some(true),
                                        last_update_time: None,
                                    },
                                )?;
                            } else {
                                transaction.create(&document_ref, &event_stream)?;
                            }
                        }

                        // イベントドキュメントの書き込み
                        for event in &events {
                            let event_id = Self::get_event_id(event);
                            let document_path =
                                Self::event_document_path(&event_stream_id, &event_id);
                            let document_ref = firestore.doc(document_path)?;
                            transaction.create(&document_ref, event)?;
                        }

                        // コールバックによる追加の更新
                        callback(transaction).await?;

                        Ok(())
                    })
                },
                TransactionOptions::default(),
            )
            .await
            .map_err(|e| RepositoryError(Box::new(e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bouzuya_firestore_client::FirestoreOptions;
    use std::fmt;

    /// テスト用のイベントストリーム ID
    #[derive(Clone, Debug)]
    struct TestEventStreamId(String);

    impl fmt::Display for TestEventStreamId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl TestEventStreamId {
        fn generate() -> Self {
            Self(uuid::Uuid::new_v4().to_string())
        }
    }

    /// テスト用のイベント
    #[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct TestEvent {
        id: String,
        at: String,
        data: String,
    }

    /// テスト用のイベントストリームドキュメント
    #[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TestEventStream {
        id: String,
        updated_at: String,
    }

    /// テスト用のリポジトリ
    struct TestRepository {
        firestore: Firestore,
    }

    impl Repository for TestRepository {
        type Event = TestEvent;
        type EventAt = String;
        type EventId = String;
        type EventStream = TestEventStream;
        type EventStreamId = TestEventStreamId;

        fn aggregate_name() -> String {
            "test".to_string()
        }

        fn firestore(&self) -> &Firestore {
            &self.firestore
        }

        fn get_event_at(event: &Self::Event) -> Self::EventAt {
            event.at.clone()
        }

        fn get_event_id(event: &Self::Event) -> Self::EventId {
            event.id.clone()
        }

        fn new_event_stream(
            event_stream_id: &Self::EventStreamId,
            events: &[Self::Event],
            _stored_event_stream: Option<Self::EventStream>,
        ) -> Self::EventStream {
            let last_event = events.last().expect("events is non-empty");
            TestEventStream {
                id: event_stream_id.to_string(),
                updated_at: last_event.at.clone(),
            }
        }
    }

    /// テスト用のリポジトリを生成する
    async fn setup_repository() -> anyhow::Result<TestRepository> {
        let firestore = Firestore::new(FirestoreOptions::default())?;
        Ok(TestRepository { firestore })
    }

    /// テスト用のイベントを生成する
    fn test_event(event_id: &str, at: &str, data: &str) -> TestEvent {
        TestEvent {
            id: event_id.to_string(),
            at: at.to_string(),
            data: data.to_string(),
        }
    }

    #[tokio::test]
    async fn test_load_events_empty() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let stream_id = TestEventStreamId::generate();

        let events = repo.load_events(&stream_id).await?;

        assert_eq!(events, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_load_single_event() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let stream_id = TestEventStreamId::generate();
        let event = test_event("evt-001", "2024-01-01T00:00:00Z", "テストデータ");

        repo.save_events(
            stream_id.clone(),
            vec![event.clone()],
            Box::new(|_| Box::pin(async { Ok(()) })),
        )
        .await?;
        let loaded = repo.load_events(&stream_id).await?;

        assert_eq!(loaded, vec![event]);
        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_load_multiple_events() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let stream_id = TestEventStreamId::generate();
        let event1 = test_event("evt-001", "2024-01-01T00:00:00Z", "最初のイベント");
        let event2 = test_event("evt-002", "2024-01-02T00:00:00Z", "二番目のイベント");

        repo.save_events(
            stream_id.clone(),
            vec![event1.clone()],
            Box::new(|_| Box::pin(async { Ok(()) })),
        )
        .await?;
        repo.save_events(
            stream_id.clone(),
            vec![event2.clone()],
            Box::new(|_| Box::pin(async { Ok(()) })),
        )
        .await?;
        let loaded = repo.load_events(&stream_id).await?;

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0], event1);
        assert_eq!(loaded[1], event2);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_events_sorted_by_at() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let stream_id = TestEventStreamId::generate();
        // at が逆順のイベントを同時に保存
        let event1 = test_event("evt-001", "2024-01-02T00:00:00Z", "後のイベント");
        let event2 = test_event("evt-002", "2024-01-01T00:00:00Z", "前のイベント");

        repo.save_events(
            stream_id.clone(),
            vec![event1.clone(), event2.clone()],
            Box::new(|_| Box::pin(async { Ok(()) })),
        )
        .await?;
        let loaded = repo.load_events(&stream_id).await?;

        assert_eq!(loaded.len(), 2);
        // at の昇順でソートされる
        assert_eq!(loaded[0], event2);
        assert_eq!(loaded[1], event1);
        Ok(())
    }

    #[tokio::test]
    async fn test_store_events_empty() -> anyhow::Result<()> {
        let repo = setup_repository().await?;
        let stream_id = TestEventStreamId::generate();

        // 空のイベントリストを保存しても何も起きない
        repo.save_events(
            stream_id.clone(),
            vec![],
            Box::new(|_| Box::pin(async { Ok(()) })),
        )
        .await?;
        let loaded = repo.load_events(&stream_id).await?;

        assert_eq!(loaded, vec![]);
        Ok(())
    }

    #[tokio::test]
    async fn test_event_collection_path() {
        let stream_id = TestEventStreamId("test-id".to_string());
        assert_eq!(
            TestRepository::event_collection_path(&stream_id),
            "aggregates/test/event_streams/test-id/events"
        );
    }

    #[tokio::test]
    async fn test_event_document_path() {
        let stream_id = TestEventStreamId("test-id".to_string());
        assert_eq!(
            TestRepository::event_document_path(&stream_id, &"evt-001".to_string()),
            "aggregates/test/event_streams/test-id/events/evt-001"
        );
    }

    #[tokio::test]
    async fn test_event_stream_document_path() {
        let stream_id = TestEventStreamId("test-id".to_string());
        assert_eq!(
            TestRepository::event_stream_document_path(&stream_id),
            "aggregates/test/event_streams/test-id"
        );
    }
}
