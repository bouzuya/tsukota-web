use std::sync::Arc;

use application::projection::AccountProjection;
use application::projection::CategoryProjection;
use application::projection::TransactionProjection;
use application::repository::EventStoreRepository;
use application::use_case::AddCategoryUseCase;
use application::use_case::AddOwnerUseCase;
use application::use_case::AddTransactionUseCase;
use application::use_case::CreateAccountUseCase;
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

/// Application state holding all use cases
pub struct AppState<R, AP, CP, TP>
where
    R: EventStoreRepository,
    AP: AccountProjection,
    CP: CategoryProjection,
    TP: TransactionProjection,
{
    // Command use cases
    pub create_account: CreateAccountUseCase<R>,
    pub update_account: UpdateAccountUseCase<R>,
    pub delete_account: DeleteAccountUseCase<R>,
    pub add_owner: AddOwnerUseCase<R>,
    pub remove_owner: RemoveOwnerUseCase<R>,
    pub add_category: AddCategoryUseCase<R>,
    pub update_category: UpdateCategoryUseCase<R>,
    pub delete_category: DeleteCategoryUseCase<R>,
    pub add_transaction: AddTransactionUseCase<R>,
    pub update_transaction: UpdateTransactionUseCase<R>,
    pub delete_transaction: DeleteTransactionUseCase<R>,

    // Query use cases
    pub list_accounts: ListAccountsUseCase<AP>,
    pub get_account: GetAccountUseCase<AP>,
    pub list_categories: ListCategoriesUseCase<AP, CP>,
    pub list_transactions: ListTransactionsUseCase<AP, TP>,
    pub export_transactions: ExportTransactionsUseCase<AP, TP>,
}

impl<R, AP, CP, TP> AppState<R, AP, CP, TP>
where
    R: EventStoreRepository + Clone,
    AP: AccountProjection + Clone,
    CP: CategoryProjection + Clone,
    TP: TransactionProjection + Clone,
{
    pub fn new(
        repository: R,
        account_projection: AP,
        category_projection: CP,
        transaction_projection: TP,
    ) -> Arc<Self> {
        Arc::new(Self {
            // Command use cases
            create_account: CreateAccountUseCase::new(repository.clone()),
            update_account: UpdateAccountUseCase::new(repository.clone()),
            delete_account: DeleteAccountUseCase::new(repository.clone()),
            add_owner: AddOwnerUseCase::new(repository.clone()),
            remove_owner: RemoveOwnerUseCase::new(repository.clone()),
            add_category: AddCategoryUseCase::new(repository.clone()),
            update_category: UpdateCategoryUseCase::new(repository.clone()),
            delete_category: DeleteCategoryUseCase::new(repository.clone()),
            add_transaction: AddTransactionUseCase::new(repository.clone()),
            update_transaction: UpdateTransactionUseCase::new(repository.clone()),
            delete_transaction: DeleteTransactionUseCase::new(repository),

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
        })
    }
}
