use domain::UserId;

/// サインイン結果
#[derive(Clone, Debug)]
pub struct SignInWithGoogleResponse {
    /// 解決された内部ユーザー ID
    pub user_id: UserId,
}
