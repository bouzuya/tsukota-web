use domain::GoogleUserId;

/// Google アカウントによるサインインリクエスト
#[derive(Clone, Debug)]
pub struct SignInWithGoogleRequest {
    /// id_token から取り出した Google sub
    pub google_user_id: GoogleUserId,
}
