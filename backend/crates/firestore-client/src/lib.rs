pub use firestore_path as path;
pub use googleapis_tonic_google_firestore_v1::google;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] E);

#[derive(Debug, thiserror::Error)]
enum E {
    #[error("build credentials")]
    BuildCredentials(#[source] google_cloud_auth::build_errors::Error),

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

    #[error("invalid value for document fields")]
    InvalidValueForDocumentFields,

    #[error("list documents")]
    ListDocuments(#[source] tonic::Status),

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
                fields: match value {
                    google::firestore::v1::Value {
                        value_type:
                            Some(google::firestore::v1::value::ValueType::MapValue(map_value)),
                    } => Ok(map_value.fields),
                    _ => Err(E::InvalidValueForDocumentFields),
                }?,
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
        let document_name = self
            .database_name
            .doc(document_path)
            .expect("DatabaseName::doc(DocumentPath) to be valid");
        let mut client = self.client().await?;
        let google::firestore::v1::GetDocumentRequest {
            consistency_selector,
            mask,
            name: _,
        } = Default::default();
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

    pub fn serialize<T>(&self, value: &T) -> Result<google::firestore::v1::Value, Error>
    where
        T: serde::Serialize,
    {
        Ok(serde_firestore_value::to_value(value).map_err(E::Serialize)?)
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
                fields: match value {
                    google::firestore::v1::Value {
                        value_type:
                            Some(google::firestore::v1::value::ValueType::MapValue(map_value)),
                    } => Ok(map_value.fields),
                    _ => Err(E::InvalidValueForDocumentFields),
                }?,
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
                fields: match value {
                    google::firestore::v1::Value {
                        value_type:
                            Some(google::firestore::v1::value::ValueType::MapValue(map_value)),
                    } => Ok(map_value.fields),
                    _ => Err(E::InvalidValueForDocumentFields),
                }?,
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
}
