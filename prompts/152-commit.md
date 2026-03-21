Migrate FirestoreUserRepository from FirestoreClient to Firestore

- Replace FirestoreClient with bouzuya_firestore_client::Firestore in
  FirestoreUserRepository
- Use run_transaction / Transaction API instead of begin_transaction /
  commit pattern
- Use collection.list_documents() + snapshot.data() for load_events
- Keep CollectionPath / DocumentPath path helpers from firestore_client::path
- Update FirestoreAccountRepository::new to accept Firestore only (no longer
  needs FirestoreClient for the embedded FirestoreUserRepository)
- Update main.rs to pass Firestore to FirestoreAccountRepository::new and
  FirestoreUserRepository::new
- Use Transaction::update (with Precondition { exists: Some(true) }) instead
  of Transaction::set for existing document updates
