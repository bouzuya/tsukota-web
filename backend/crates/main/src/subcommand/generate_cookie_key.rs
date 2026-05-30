use api::CookieKey;

/// 新しい署名鍵 (64 byte) を生成し hex 化して標準出力に書き出す。
/// 出力は `COOKIE_SIGNING_SECRET` にそのまま設定できる形式。
pub(super) fn run() {
    let key = CookieKey::generate();
    println!("{}", hex::encode(key.master()));
}
