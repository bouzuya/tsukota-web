Add common Repository trait for infra layer

Add a shared `Repository` trait to `backend/crates/infra/src/repository.rs`
that defines the common interface for Firestore-based event sourcing
repositories. The trait includes associated types for Event, EventStream,
and EventStreamId, along with methods for document path construction,
event loading, and event storing. This trait is not yet applied to the
existing repository implementations.
