Apply Repository trait to FirestoreAccountRepository

- Change Repository::new_event_stream to accept Option<Self::EventStream>
  for stored event stream to support update scenarios (Account needs to
  merge owners from existing state)
- Implement Repository trait for FirestoreAccountRepository with
  event_stream_collection_path, get_event_at, get_event_id, and
  new_event_stream (handles owner tracking for both create and update)
- Delegate AccountRepository::load_events and save_events to Repository
  trait methods, using the callback for query collection writes
- Remove duplicated internal error type E and manual Firestore operations
  (load_events_impl, save_events_impl, build_aggregate_writes_in_tx)
- Update FirestoreUserRepository, FirestoreDeviceRepository, and test
  implementations for the new new_event_stream signature
