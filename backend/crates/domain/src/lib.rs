mod aggregates;
mod value_objects;

// 後方互換性のためのエイリアス
pub mod account {
    pub use crate::aggregates::Account;
    pub use crate::aggregates::AccountCommand;
    pub use crate::aggregates::AccountError;
    pub use crate::aggregates::AccountEvent;
    pub use crate::aggregates::AccountEventCommonProps;
    pub use crate::aggregates::Category;
    pub use crate::aggregates::Transaction;
    pub use crate::aggregates::TransactionProps;
    pub use crate::value_objects::AccountId;
    pub use crate::value_objects::CategoryId;
    pub use crate::value_objects::DeviceId;
    pub use crate::value_objects::DeviceSecret;
    pub use crate::value_objects::ParseAccountIdError;
    pub use crate::value_objects::ParseCategoryIdError;
    pub use crate::value_objects::ParseDeviceIdError;
    pub use crate::value_objects::ParseDeviceSecretError;
    pub use crate::value_objects::ParseTransactionIdError;
    pub use crate::value_objects::ParseUserIdError;
    pub use crate::value_objects::TransactionId;
    pub use crate::value_objects::UserId;
}

pub use self::aggregates::Account;
pub use self::aggregates::AccountCommand;
pub use self::aggregates::AccountError;
pub use self::aggregates::AccountEvent;
pub use self::aggregates::AccountEventCommonProps;
pub use self::aggregates::Category;
pub use self::aggregates::Transaction;
pub use self::aggregates::TransactionProps;
pub use self::value_objects::AccountId;
pub use self::value_objects::CategoryId;
pub use self::value_objects::DeviceId;
pub use self::value_objects::DeviceSecret;
pub use self::value_objects::ParseAccountIdError;
pub use self::value_objects::ParseCategoryIdError;
pub use self::value_objects::ParseDeviceIdError;
pub use self::value_objects::ParseDeviceSecretError;
pub use self::value_objects::ParseTransactionIdError;
pub use self::value_objects::ParseUserIdError;
pub use self::value_objects::TransactionId;
pub use self::value_objects::UserId;
