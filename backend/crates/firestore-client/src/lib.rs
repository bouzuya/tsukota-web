pub use googleapis_tonic_google_firestore_v1::google;

pub struct FirestoreClient {
    channel: tonic::transport::Channel,
    credentials: google_cloud_auth::credentials::Credentials,
}

impl FirestoreClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn client(
        &self,
    ) -> Result<
        google::firestore::v1::firestore_client::FirestoreClient<
            tonic::service::interceptor::InterceptedService<
                tonic::transport::Channel,
                impl FnMut(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>,
            >,
        >,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let cacheable_headers = self.credentials.headers(http::Extensions::new()).await?;
        let header_map = match cacheable_headers {
            google_cloud_auth::credentials::CacheableResource::New { data, .. } => data,
            google_cloud_auth::credentials::CacheableResource::NotModified => {
                return Err("NotModified response without cached data".into());
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
