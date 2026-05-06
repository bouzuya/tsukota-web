use domain::GoogleUserId;

/// Google アカウントによるサインアップリクエスト
#[derive(Clone, Debug)]
pub struct SignUpWithGoogleRequest {
    /// id_token から取り出した Google sub
    pub google_user_id: GoogleUserId,
}
