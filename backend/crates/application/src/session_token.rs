/// 作成エラー型（object-safe のための Box 化）
pub type CreatorError = Box<dyn std::error::Error + Send + Sync>;

/// 検証エラー型（object-safe のための Box 化）
pub type VerifierError = Box<dyn std::error::Error + Send + Sync>;

/// セッショントークンを作成するための trait
pub trait SessionTokenCreator: Send + Sync {
    /// 現在時刻を UNIX エポックからの秒数で返す
    fn now(&self) -> Result<u64, CreatorError>;

    /// 指定された UID に対するセッショントークンを作成する
    ///
    /// # Arguments
    ///
    /// * `uid` - ユーザー識別子 (1-128 文字)
    /// * `now` - 現在時刻 (UNIX エポックからの秒数)
    fn create(&self, uid: &str, now: u64) -> Result<String, CreatorError>;
}

/// トークンを検証して UID を取得するための trait
pub trait SessionTokenVerifier: Send + Sync {
    /// トークンを検証して UID を取得する
    ///
    /// # Arguments
    ///
    /// * `token` - 検証するトークン
    ///
    /// # Returns
    ///
    /// トークンに含まれる UID、または検証エラー
    fn verify(&self, token: &str) -> Result<String, VerifierError>;
}
