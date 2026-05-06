use domain::UserId;

/// サインアップ結果
#[derive(Clone, Debug)]
pub struct SignUpWithGoogleResponse {
    /// 新規発行された内部ユーザー ID
    pub user_id: UserId,
}
