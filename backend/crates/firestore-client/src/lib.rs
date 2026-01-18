pub use firestore_path as path;
pub use googleapis_tonic_google_firestore_v1::google;

pub struct Transaction(pub(crate) Vec<u8>);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] E);

#[derive(Debug, thiserror::Error)]
enum E {
    #[error("begin transaction")]
    BeginTransaction(#[source] tonic::Status),

    #[error("build credentials")]
    BuildCredentials(#[source] google_cloud_auth::build_errors::Error),

    #[error("commit")]
    Commit(#[source] tonic::Status),

    #[error("connect")]
    Connect(#[source] tonic::transport::Error),

    #[error("create document")]
    CreateDocument(#[source] tonic::Status),

    #[error("credentials")]
    Credentials(#[source] google_cloud_auth::errors::CredentialsError),

    #[error("credentials not modified")]
    CredentialsNotModified,

    #[error("deserialize")]
    Deserialize(#[source] serde_firestore_value::Error),

    #[error("get document")]
    GetDocument(#[source] tonic::Status),

    #[error("list documents")]
    ListDocuments(#[source] tonic::Status),

    #[error("rollback")]
    Rollback(#[source] tonic::Status),

    #[error("serialize")]
    Serialize(#[source] serde_firestore_value::Error),

    #[error("set document")]
    SetDocument(#[source] tonic::Status),

    #[error("tls config")]
    TlsConfig(#[source] tonic::transport::Error),

    #[error("update document")]
    UpdateDocument(#[source] tonic::Status),
}

#[derive(Clone)]
pub struct FirestoreClient {
    channel: tonic::transport::Channel,
    credentials: Option<google_cloud_auth::credentials::Credentials>,
    database_name: path::DatabaseName,
}

/// Functions
impl FirestoreClient {
    pub async fn connect(database_name: path::DatabaseName) -> Result<Self, Error> {
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())
            .map_err(E::TlsConfig)?
            .connect()
            .await
            .map_err(E::Connect)?;
        let credentials = google_cloud_auth::credentials::Builder::default()
            .with_scopes(["https://www.googleapis.com/auth/datastore"])
            .build()
            .map_err(E::BuildCredentials)?;
        Ok(Self {
            channel,
            credentials: Some(credentials),
            database_name,
        })
    }

    pub async fn connect_with_emulator() -> Result<Self, Error> {
        let database_name = path::DatabaseName::from_project_id("demo-project").unwrap();
        let channel = tonic::transport::Channel::from_static("http://firebase:8080")
            .connect()
            .await
            .map_err(E::Connect)?;
        Ok(Self {
            channel,
            credentials: None,
            database_name,
        })
    }
}

/// Methods
impl FirestoreClient {
    pub async fn begin_transaction(&self) -> Result<Transaction, Error> {
        let mut client = self.client().await?;
        let google::firestore::v1::BeginTransactionRequest {
            database: _,
            options,
        } = Default::default();
        let request = google::firestore::v1::BeginTransactionRequest {
            database: self.database_name.to_string(),
            options,
        };
        Ok(Transaction(
            client
                .begin_transaction(request)
                .await
                .map_err(E::BeginTransaction)?
                .into_inner()
                .transaction,
        ))
    }

    pub async fn commit(
        &self,
        Transaction(transaction): &Transaction,
        writes: Vec<google::firestore::v1::Write>,
    ) -> Result<google::firestore::v1::CommitResponse, Error> {
        let mut client = self.client().await?;
        let request = google::firestore::v1::CommitRequest {
            database: self.database_name.to_string(),
            writes,
            transaction: transaction.to_owned(),
        };
        Ok(client
            .commit(request)
            .await
            .map_err(E::Commit)?
            .into_inner())
    }

    pub async fn create_document(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> Result<google::firestore::v1::Document, Error> {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::CreateDocumentRequest {
            collection_id: _,
            document: _,
            document_id: _,
            mask,
            parent: _,
        } = Default::default();
        let request = google::firestore::v1::CreateDocumentRequest {
            collection_id: document_name.collection_id().to_string(),
            document_id: document_name.document_id().to_string(),
            mask,
            parent: document_name
                .parent_document_name()
                .map(|parent| parent.to_string())
                .unwrap_or_else(|| document_name.root_document_name().to_string()),
            document: Some(google::firestore::v1::Document {
                name: String::new(),
                fields: Self::extract_fields(value),
                create_time: None,
                update_time: None,
            }),
        };
        Ok(client
            .create_document(request)
            .await
            .map_err(E::CreateDocument)?
            .into_inner())
    }

    pub fn deserialize<T>(
        &self,
        fields: std::collections::HashMap<String, google::firestore::v1::Value>,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let value = google::firestore::v1::Value {
            value_type: Some(google::firestore::v1::value::ValueType::MapValue(
                google::firestore::v1::MapValue { fields },
            )),
        };
        Ok(serde_firestore_value::from_value(&value).map_err(E::Deserialize)?)
    }

    pub async fn get_document(
        &self,
        document_path: path::DocumentPath,
    ) -> Result<Option<google::firestore::v1::Document>, Error> {
        self.get_document_impl(document_path, None).await
    }

    pub async fn get_document_with_tx(
        &self,
        document_path: path::DocumentPath,
        transaction: &Transaction,
    ) -> Result<Option<google::firestore::v1::Document>, Error> {
        self.get_document_impl(document_path, Some(transaction))
            .await
    }

    pub async fn list_documents(
        &self,
        collection_path: path::CollectionPath,
        page_token: Option<String>,
    ) -> Result<google::firestore::v1::ListDocumentsResponse, Error> {
        let collection_name = self
            .database_name
            .collection(collection_path)
            .expect("DatabaseName::collection(CollectionPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::ListDocumentsRequest {
            collection_id: _,
            consistency_selector,
            mask,
            order_by,
            page_size,
            page_token: _,
            parent: _,
            show_missing,
        } = Default::default();
        let request = google::firestore::v1::ListDocumentsRequest {
            collection_id: collection_name.collection_id().to_string(),
            consistency_selector,
            mask,
            order_by,
            page_size,
            page_token: page_token.unwrap_or_default(),
            parent: collection_name
                .parent()
                .map(|parent| parent.to_string())
                .unwrap_or_else(|| self.database_name.root_document_name().to_string()),
            show_missing,
        };
        Ok(client
            .list_documents(request)
            .await
            .map_err(E::ListDocuments)?
            .into_inner())
    }

    pub async fn rollback(&self, Transaction(transaction): &Transaction) -> Result<(), Error> {
        let mut client = self.client().await?;
        let request = google::firestore::v1::RollbackRequest {
            database: self.database_name.to_string(),
            transaction: transaction.to_owned(),
        };
        client.rollback(request).await.map_err(E::Rollback)?;
        Ok(())
    }

    pub fn serialize<T>(&self, value: &T) -> Result<google::firestore::v1::Value, Error>
    where
        T: serde::Serialize,
    {
        Ok(serde_firestore_value::to_value(value).map_err(E::Serialize)?)
    }

    pub fn build_create_write(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> google::firestore::v1::Write {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        google::firestore::v1::Write {
            current_document: Some(google::firestore::v1::Precondition {
                condition_type: Some(google::firestore::v1::precondition::ConditionType::Exists(
                    false,
                )),
            }),
            operation: Some(google::firestore::v1::write::Operation::Update(
                google::firestore::v1::Document {
                    create_time: None,
                    fields: Self::extract_fields(value),
                    name: document_name.to_string(),
                    update_time: None,
                },
            )),
            update_mask: None,
            update_transforms: vec![],
        }
    }

    pub fn build_delete_write(
        &self,
        document_path: path::DocumentPath,
    ) -> google::firestore::v1::Write {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        google::firestore::v1::Write {
            current_document: Some(google::firestore::v1::Precondition {
                condition_type: Some(google::firestore::v1::precondition::ConditionType::Exists(
                    true,
                )),
            }),
            operation: Some(google::firestore::v1::write::Operation::Delete(
                document_name.to_string(),
            )),
            update_mask: None,
            update_transforms: vec![],
        }
    }

    pub fn build_set_write(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> google::firestore::v1::Write {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        google::firestore::v1::Write {
            current_document: None,
            operation: Some(google::firestore::v1::write::Operation::Update(
                google::firestore::v1::Document {
                    create_time: None,
                    fields: Self::extract_fields(value),
                    name: document_name.to_string(),
                    update_time: None,
                },
            )),
            update_mask: None,
            update_transforms: vec![],
        }
    }

    pub fn build_update_write(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> google::firestore::v1::Write {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        google::firestore::v1::Write {
            current_document: Some(google::firestore::v1::Precondition {
                condition_type: Some(google::firestore::v1::precondition::ConditionType::Exists(
                    true,
                )),
            }),
            operation: Some(google::firestore::v1::write::Operation::Update(
                google::firestore::v1::Document {
                    create_time: None,
                    fields: Self::extract_fields(value),
                    name: document_name.to_string(),
                    update_time: None,
                },
            )),
            update_mask: None,
            update_transforms: vec![],
        }
    }

    pub async fn set_document(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> Result<google::firestore::v1::Document, Error> {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::UpdateDocumentRequest {
            current_document: _,
            document: _,
            mask,
            update_mask,
        } = Default::default();
        let request = google::firestore::v1::UpdateDocumentRequest {
            current_document: None,
            mask,
            update_mask,
            document: Some(google::firestore::v1::Document {
                create_time: None,
                fields: Self::extract_fields(value),
                name: document_name.to_string(),
                update_time: None,
            }),
        };
        Ok(client
            .update_document(request)
            .await
            .map_err(E::SetDocument)?
            .into_inner())
    }

    pub async fn update_document(
        &self,
        document_path: path::DocumentPath,
        value: google::firestore::v1::Value,
    ) -> Result<google::firestore::v1::Document, Error> {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::UpdateDocumentRequest {
            current_document: _,
            document: _,
            mask,
            update_mask,
        } = Default::default();
        let request = google::firestore::v1::UpdateDocumentRequest {
            current_document: Some(google::firestore::v1::Precondition {
                condition_type: Some(google::firestore::v1::precondition::ConditionType::Exists(
                    true,
                )),
            }),
            mask,
            update_mask,
            document: Some(google::firestore::v1::Document {
                create_time: None,
                fields: Self::extract_fields(value),
                name: document_name.to_string(),
                update_time: None,
            }),
        };
        Ok(client
            .update_document(request)
            .await
            .map_err(E::UpdateDocument)?
            .into_inner())
    }

    async fn client(
        &self,
    ) -> Result<
        google::firestore::v1::firestore_client::FirestoreClient<
            tonic::service::interceptor::InterceptedService<
                tonic::transport::Channel,
                impl FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
            >,
        >,
        Error,
    > {
        let metadata = match &self.credentials {
            Some(credentials) => {
                let cacheable_headers = credentials
                    .headers(http::Extensions::new())
                    .await
                    .map_err(E::Credentials)?;
                let header_map = match cacheable_headers {
                    google_cloud_auth::credentials::CacheableResource::New { data, .. } => data,
                    google_cloud_auth::credentials::CacheableResource::NotModified => {
                        return Err(E::CredentialsNotModified)?;
                    }
                };
                tonic::metadata::MetadataMap::from_headers(header_map)
            }
            None => tonic::metadata::MetadataMap::new(),
        };

        let firestore_client =
            google::firestore::v1::firestore_client::FirestoreClient::with_interceptor(
                self.channel.clone(),
                move |mut request: tonic::Request<()>| {
                    for key_and_value in metadata.iter() {
                        match key_and_value {
                            tonic::metadata::KeyAndValueRef::Ascii(key, value) => {
                                request.metadata_mut().insert(key, value.clone());
                            }
                            tonic::metadata::KeyAndValueRef::Binary(key, value) => {
                                request.metadata_mut().insert_bin(key, value.clone());
                            }
                        }
                    }
                    Ok(request)
                },
            );

        Ok(firestore_client)
    }

    fn extract_fields(
        value: google::firestore::v1::Value,
    ) -> std::collections::HashMap<String, google::firestore::v1::Value> {
        match value {
            google::firestore::v1::Value {
                value_type: Some(google::firestore::v1::value::ValueType::MapValue(map_value)),
            } => map_value.fields,
            _ => panic!("value must be a MapValue"),
        }
    }

    async fn get_document_impl(
        &self,
        document_path: path::DocumentPath,
        transaction: Option<&Transaction>,
    ) -> Result<Option<google::firestore::v1::Document>, Error> {
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::GetDocumentRequest {
            consistency_selector: _,
            mask,
            name: _,
        } = Default::default();
        let consistency_selector = transaction.map(|Transaction(tx)| {
            google::firestore::v1::get_document_request::ConsistencySelector::Transaction(
                tx.to_owned(),
            )
        });
        let request = google::firestore::v1::GetDocumentRequest {
            consistency_selector,
            mask,
            name: document_name.to_string(),
        };
        let result = client.get_document(request).await;
        match result {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(status) => Err(E::GetDocument(status))?,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[tokio::test]
    async fn test_crud() -> anyhow::Result<()> {
        let client = FirestoreClient::connect_with_emulator().await?;

        let document_id =
            <path::DocumentId as std::str::FromStr>::from_str(&uuid::Uuid::new_v4().to_string())?;
        let document_path =
            <path::CollectionPath as std::str::FromStr>::from_str("test_collection")?
                .doc(document_id.clone())?;

        assert!(client.get_document(document_path.clone()).await?.is_none());

        // create_document
        #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        struct TestDocumentData {
            n: i64,
            s: String,
        }
        let document_data1 = TestDocumentData {
            n: 123,
            s: "abc".to_owned(),
        };
        let serialized = client.serialize(&document_data1)?;
        let created = client
            .create_document(document_path.clone(), serialized.clone())
            .await?;
        assert_eq!(
            <path::DocumentName as std::str::FromStr>::from_str(&created.name)?.document_id(),
            &document_id
        );
        let deserialized = client.deserialize::<TestDocumentData>(created.fields)?;
        assert_eq!(deserialized, document_data1);

        // get_document (created)
        let fetched = client.get_document(document_path.clone()).await?;
        let fetched = fetched.context("document not found")?;
        assert_eq!(fetched.name, created.name);
        let deserialized = client.deserialize::<TestDocumentData>(fetched.fields)?;
        assert_eq!(deserialized, document_data1);

        let document_data2 = TestDocumentData {
            n: 456,
            s: "def".to_owned(),
        };
        let serialized = client.serialize(&document_data2)?;
        let updated = client
            .update_document(document_path.clone(), serialized.clone())
            .await?;
        assert_eq!(
            <path::DocumentName as std::str::FromStr>::from_str(&updated.name)?.document_id(),
            &document_id
        );
        let deserialized = client.deserialize::<TestDocumentData>(updated.fields)?;
        assert_eq!(deserialized, document_data2);

        // get_document (updated)
        let fetched = client.get_document(document_path.clone()).await?;
        let fetched = fetched.context("document not found")?;
        assert_eq!(fetched.name, updated.name);
        let deserialized = client.deserialize::<TestDocumentData>(fetched.fields)?;
        assert_eq!(deserialized, document_data2);

        // TODO: delete

        Ok(())
    }

    #[tokio::test]
    async fn test_begin_transaction_and_commit() -> anyhow::Result<()> {
        let client = FirestoreClient::connect_with_emulator().await?;

        let document_id =
            <path::DocumentId as std::str::FromStr>::from_str(&uuid::Uuid::new_v4().to_string())?;
        let document_path =
            <path::CollectionPath as std::str::FromStr>::from_str("test_collection")?
                .doc(document_id.clone())?;

        // Begin transaction
        let transaction = client.begin_transaction().await?;

        // Create a document via commit
        #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        struct TestDocumentData {
            n: i64,
            s: String,
        }
        let document_data = TestDocumentData {
            n: 789,
            s: "xyz".to_owned(),
        };
        let serialized = client.serialize(&document_data)?;
        let writes = vec![client.build_set_write(document_path.clone(), serialized)];

        let commit_response = client.commit(&transaction, writes).await?;
        assert!(commit_response.commit_time.is_some());

        // Verify document was created
        let fetched = client.get_document(document_path).await?;
        let fetched = fetched.context("document not found after commit")?;
        let deserialized = client.deserialize::<TestDocumentData>(fetched.fields)?;
        assert_eq!(deserialized, document_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_begin_transaction_and_rollback() -> anyhow::Result<()> {
        let client = FirestoreClient::connect_with_emulator().await?;

        // Begin transaction
        let transaction = client.begin_transaction().await?;

        // Rollback transaction (no writes)
        client.rollback(&transaction).await?;

        Ok(())
    }
}
