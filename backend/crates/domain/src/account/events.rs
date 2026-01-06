/// イベントの共通プロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AccountEventCommonProps {
    pub account_id: String,
    pub at: String,
    pub id: String,
    pub protocol_version: u32,
}

/// 取引のプロパティ
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TransactionProps {
    pub amount: String,
    pub category_id: String,
    pub comment: String,
    pub date: String,
}

/// アカウント集約に対するイベント
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AccountEvent {
    /// アカウントが作成された
    #[serde(rename_all = "camelCase")]
    AccountCreated {
        name: String,
        owners: Vec<String>,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// アカウントが削除された
    #[serde(rename_all = "camelCase")]
    AccountDeleted {
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// アカウント名が変更された
    #[serde(rename_all = "camelCase")]
    AccountUpdated {
        name: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// カテゴリが追加された
    #[serde(rename_all = "camelCase")]
    CategoryAdded {
        category_id: String,
        name: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// カテゴリが削除された
    #[serde(rename_all = "camelCase")]
    CategoryDeleted {
        category_id: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// カテゴリ名が変更された
    #[serde(rename_all = "camelCase")]
    CategoryUpdated {
        category_id: String,
        name: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// オーナーが追加された
    #[serde(rename_all = "camelCase")]
    OwnerAdded {
        owner: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// オーナーが削除された
    #[serde(rename_all = "camelCase")]
    OwnerRemoved {
        owner: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// 取引が追加された
    #[serde(rename_all = "camelCase")]
    TransactionAdded {
        transaction_id: String,
        #[serde(flatten)]
        props: TransactionProps,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// 取引が削除された
    #[serde(rename_all = "camelCase")]
    TransactionDeleted {
        transaction_id: String,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },

    /// 取引が更新された
    #[serde(rename_all = "camelCase")]
    TransactionUpdated {
        transaction_id: String,
        #[serde(flatten)]
        props: TransactionProps,
        #[serde(flatten)]
        common: AccountEventCommonProps,
    },
}
