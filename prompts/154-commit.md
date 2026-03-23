Migrate FirestoreDeviceRepository from FirestoreClient to Firestore

- Replace `FirestoreClient` with `bouzuya_firestore_client::Firestore` in
  `firestore_device_repository.rs`
- Rewrite `load_events_impl` to use `collection.list_documents()` and
  `document_ref.get()` via the new Firestore API
- Rewrite `save_events_impl` to use `firestore.run_transaction` with
  `transaction.create` / `transaction.update`
- Preserve `firestore_client::path::{CollectionPath, DocumentPath}` path
  helpers as required
- Update `main.rs` to pass `firestore` (Firestore) instead of
  `firestore_client` (FirestoreClient) to `FirestoreDeviceRepository::new`
