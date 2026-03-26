Migrate FirestoreProjection from FirestoreClient to Firestore

- Replace FirestoreClient with bouzuya_firestore_client::Firestore in
  FirestoreProjection
- Update load_events to use collection/list_documents/get_all/data pattern
- Update list_accounts to use doc/get_all/data pattern for user document
- Update error enum to use bouzuya_firestore_client::Error variants
- Remove unused FirestoreClient creation from main.rs
- Keep firestore_client::path::{CollectionPath, DocumentPath} imports
