pub use firestore_path as path;
pub use googleapis_tonic_google_firestore_v1::google;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("credentials error: {0}")]
    Credentials(#[from] google_cloud_auth::errors::CredentialsError),

    #[error("build credentials error: {0}")]
    BuildCredentials(#[from] google_cloud_auth::build_errors::Error),

    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("grpc status error: {0}")]
    Status(#[from] tonic::Status),

    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_firestore_value::Error),

    #[error("credentials not modified")]
    CredentialsNotModified,
}

#[derive(Clone)]
pub struct FirestoreClient {
    channel: tonic::transport::Channel,
    credentials: google_cloud_auth::credentials::Credentials,
}

impl FirestoreClient {
    pub async fn connect() -> Result<Self, Error> {
        let channel = tonic::transport::Channel::from_static("https://firestore.googleapis.com")
            .tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())?
            .connect()
            .await?;
        let credentials = google_cloud_auth::credentials::Builder::default().build()?;
        Ok(Self {
            channel,
            credentials,
        })
    }

    pub async fn deserialize<T>(
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
        Ok(serde_firestore_value::from_value(&value)?)
    }

    pub async fn get_document(
        &self,
        document_path: &path::DocumentPath,
    ) -> Result<Option<google::firestore::v1::Document>, Error> {
        let mut client = self.client().await?;
        let google::firestore::v1::GetDocumentRequest {
            consistency_selector,
            mask,
            name: _,
        } = Default::default();
        let request = google::firestore::v1::GetDocumentRequest {
            consistency_selector,
            mask,
            name: document_path.to_string(),
        };
        let result = client.get_document(request).await;
        match result {
            Ok(response) => Ok(Some(response.into_inner())),
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(status) => Err(Error::Status(status)),
        }
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
        let cacheable_headers = self.credentials.headers(http::Extensions::new()).await?;
        let header_map = match cacheable_headers {
            google_cloud_auth::credentials::CacheableResource::New { data, .. } => data,
            google_cloud_auth::credentials::CacheableResource::NotModified => {
                return Err(Error::CredentialsNotModified);
            }
        };
        let metadata = tonic::metadata::MetadataMap::from_headers(header_map);

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
