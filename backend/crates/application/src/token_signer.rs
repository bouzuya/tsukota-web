/// 署名エラー型（object-safe のための Box 化）
pub type SignerError = Box<dyn std::error::Error + Send + Sync>;

/// Firebase カスタムトークンを署名するための trait
pub trait TokenSigner: Send + Sync {
    /// 現在時刻を UNIX エポックからの秒数で返す
    fn now(&self) -> Result<u64, SignerError>;

    /// 指定された UID に対する Firebase カスタムトークンを署名する
    ///
    /// # Arguments
    ///
    /// * `uid` - ユーザー識別子 (1-128 文字)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    fn sign(&self, uid: &str, now: u64) -> Result<String, SignerError>;
}
