/// Firebase カスタムトークンを署名するための trait
pub trait TokenSigner: Clone + Send + Sync {
    /// 署名エラー型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 現在時刻を UNIX エポックからの秒数で返す
    fn now(&self) -> Result<u64, Self::Error>;

    /// 指定された UID に対する Firebase カスタムトークンを署名する
    ///
    /// # Arguments
    ///
    /// * `uid` - ユーザー識別子 (1-128 文字)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    fn sign(&self, uid: &str, now: u64) -> Result<String, Self::Error>;
}
