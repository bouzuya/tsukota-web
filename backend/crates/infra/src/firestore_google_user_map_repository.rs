use application::error::ApplicationError;
use application::error::GoogleUserError;
use application::repository::GoogleUserMapRepository;
use async_trait::async_trait;
use bouzuya_firestore_client::Firestore;
use bouzuya_firestore_client::TransactionOptions;
use domain::GoogleUserId;
use domain::UserId;

use crate::schema::QueryGoogleUserMapDocumentData;

/// Firestore の `google_user_map/{sub}` ドキュメントとして
/// Google sub と内部 UserId の対応を保管するリポジトリ
#[derive(Clone)]
pub struct FirestoreGoogleUserMapRepository {
    firestore: Firestore,
}

impl FirestoreGoogleUserMapRepository {
    pub fn new(firestore: Firestore) -> Self {
        Self { firestore }
    }

    fn document_path(google_user_id: &GoogleUserId) -> String {
        format!("google_user_map/{}", google_user_id)
    }
}

#[async_trait]
impl GoogleUserMapRepository for FirestoreGoogleUserMapRepository {
    async fn find_user_id_by_google_user_id(
        &self,
        google_user_id: &GoogleUserId,
    ) -> Result<Option<UserId>, ApplicationError> {
        let document_path = Self::document_path(google_user_id);
        let firestore = self.firestore.clone();
        let result = self
            .firestore
            .run_transaction(
                move |transaction| {
                    Box::pin(async move {
                        let document_ref = firestore.doc(document_path)?;
                        let snapshot = transaction.get(&document_ref).await?;
                        snapshot
                            .data::<QueryGoogleUserMapDocumentData>()
                            .transpose()
                    })
                },
                TransactionOptions::default(),
            )
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        match result {
            None => Ok(None),
            Some(doc) => {
                let user_id = doc
                    .user_id
                    .parse::<UserId>()
                    .map_err(|e| ApplicationError::Repository(format!("invalid UserId: {e}")))?;
                Ok(Some(user_id))
            }
        }
    }

    async fn save(
        &self,
        google_user_id: &GoogleUserId,
        user_id: &UserId,
    ) -> Result<(), ApplicationError> {
        let document_path = Self::document_path(google_user_id);
        let doc = QueryGoogleUserMapDocumentData {
            google_user_id: google_user_id.to_string(),
            user_id: user_id.to_string(),
        };
        let firestore = self.firestore.clone();
        self.firestore
            .run_transaction(
                move |transaction| {
                    let doc = doc.clone();
                    Box::pin(async move {
                        let document_ref = firestore.doc(document_path)?;
                        transaction.create(&document_ref, &doc)?;
                        Ok(())
                    })
                },
                TransactionOptions::default(),
            )
            .await
            .map_err(|e| {
                // Firestore からの ALREADY_EXISTS は AlreadyRegistered として上位へ伝える
                let msg = e.to_string();
                if msg.contains("ALREADY_EXISTS") || msg.contains("already exists") {
                    ApplicationError::GoogleUser(GoogleUserError::AlreadyRegistered)
                } else {
                    ApplicationError::Repository(msg)
                }
            })?;
        Ok(())
    }
}
