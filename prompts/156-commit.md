Replace CollectionPath/DocumentPath with String in infra crate

Remove dependency on firestore_client::path types by replacing
CollectionPath and DocumentPath with plain String in all path
generation methods across infra repositories and projection.
