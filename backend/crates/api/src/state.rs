use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::AccountRepository;
use application::repository::DeviceRepository;
use application::repository::UserRepository;
use application::token_signer::TokenSigner;
use application::use_case::AddCategoryUseCase;
use application::use_case::AddOwnerUseCase;
use application::use_case::AddTransactionUseCase;
use application::use_case::CreateAccountUseCase;
use application::use_case::CreateCustomTokenUseCase;
use application::use_case::DeleteAccountUseCase;
use application::use_case::DeleteCategoryUseCase;
use application::use_case::DeleteTransactionUseCase;
use application::use_case::ExportTransactionsUseCase;
use application::use_case::GetAccountUseCase;
use application::use_case::ListAccountsUseCase;
use application::use_case::ListCategoriesUseCase;
use application::use_case::ListTransactionsUseCase;
use application::use_case::RemoveOwnerUseCase;
use application::use_case::UpdateAccountUseCase;
use application::use_case::UpdateCategoryUseCase;
use application::use_case::UpdateTransactionUseCase;
use axum::extract::FromRef;

/// Application state holding all use cases
#[derive(Clone)]
pub struct AppState {
    // Command use cases
    pub create_account: CreateAccountUseCase,
    pub update_account: UpdateAccountUseCase,
    pub delete_account: DeleteAccountUseCase,
    pub add_owner: AddOwnerUseCase,
    pub remove_owner: RemoveOwnerUseCase,
    pub add_category: AddCategoryUseCase,
    pub update_category: UpdateCategoryUseCase,
    pub delete_category: DeleteCategoryUseCase,
    pub add_transaction: AddTransactionUseCase,
    pub update_transaction: UpdateTransactionUseCase,
    pub delete_transaction: DeleteTransactionUseCase,
    pub create_custom_token: CreateCustomTokenUseCase,

    // Query use cases
    pub list_accounts: ListAccountsUseCase,
    pub get_account: GetAccountUseCase,
    pub list_categories: ListCategoriesUseCase,
    pub list_transactions: ListTransactionsUseCase,
    pub export_transactions: ExportTransactionsUseCase,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        account_projection: Arc<dyn AccountProjection>,
        category_projection: Arc<dyn CategoryProjection>,
        transaction_projection: Arc<dyn TransactionProjection>,
        device_repository: Arc<dyn DeviceRepository>,
        signer: Arc<dyn TokenSigner>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            // Command use cases
            create_account: CreateAccountUseCase::new(account_repository.clone()),
            update_account: UpdateAccountUseCase::new(account_repository.clone()),
            delete_account: DeleteAccountUseCase::new(account_repository.clone()),
            add_owner: AddOwnerUseCase::new(account_repository.clone()),
            remove_owner: RemoveOwnerUseCase::new(account_repository.clone()),
            add_category: AddCategoryUseCase::new(account_repository.clone()),
            update_category: UpdateCategoryUseCase::new(account_repository.clone()),
            delete_category: DeleteCategoryUseCase::new(account_repository.clone()),
            add_transaction: AddTransactionUseCase::new(account_repository.clone()),
            update_transaction: UpdateTransactionUseCase::new(account_repository.clone()),
            delete_transaction: DeleteTransactionUseCase::new(account_repository),
            create_custom_token: CreateCustomTokenUseCase::new(
                device_repository,
                signer,
                user_repository,
            ),

            // Query use cases
            list_accounts: ListAccountsUseCase::new(account_projection.clone()),
            get_account: GetAccountUseCase::new(account_projection.clone()),
            list_categories: ListCategoriesUseCase::new(
                account_projection.clone(),
                category_projection,
            ),
            list_transactions: ListTransactionsUseCase::new(
                account_projection.clone(),
                transaction_projection.clone(),
            ),
            export_transactions: ExportTransactionsUseCase::new(
                account_projection,
                transaction_projection,
            ),
        }
    }
}

// FromRef implementations for each use case

impl FromRef<AppState> for CreateAccountUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.create_account.clone()
    }
}

impl FromRef<AppState> for UpdateAccountUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.update_account.clone()
    }
}

impl FromRef<AppState> for DeleteAccountUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.delete_account.clone()
    }
}

impl FromRef<AppState> for AddOwnerUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.add_owner.clone()
    }
}

impl FromRef<AppState> for RemoveOwnerUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.remove_owner.clone()
    }
}

impl FromRef<AppState> for AddCategoryUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.add_category.clone()
    }
}

impl FromRef<AppState> for UpdateCategoryUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.update_category.clone()
    }
}

impl FromRef<AppState> for DeleteCategoryUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.delete_category.clone()
    }
}

impl FromRef<AppState> for AddTransactionUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.add_transaction.clone()
    }
}

impl FromRef<AppState> for UpdateTransactionUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.update_transaction.clone()
    }
}

impl FromRef<AppState> for DeleteTransactionUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.delete_transaction.clone()
    }
}

impl FromRef<AppState> for CreateCustomTokenUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.create_custom_token.clone()
    }
}

impl FromRef<AppState> for ListAccountsUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.list_accounts.clone()
    }
}

impl FromRef<AppState> for GetAccountUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.get_account.clone()
    }
}

impl FromRef<AppState> for ListCategoriesUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.list_categories.clone()
    }
}

impl FromRef<AppState> for ListTransactionsUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.list_transactions.clone()
    }
}

impl FromRef<AppState> for ExportTransactionsUseCase {
    fn from_ref(state: &AppState) -> Self {
        state.export_transactions.clone()
    }
}
