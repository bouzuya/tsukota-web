// Converters between API types (snake_case) and frontend types (camelCase)

import type {
  ApiUser,
  ApiAccount,
  ApiCategory,
  ApiTransaction,
  ApiPaginatedResponse,
  ApiCreateAccountResponse,
  ApiAddCategoryResponse,
  ApiAddTransactionResponse,
} from './apiTypes';

import type {
  User,
  Account,
  Category,
  Transaction,
  PaginatedResponse,
  CreateAccountResponse,
  AddCategoryResponse,
  AddTransactionResponse,
} from './types';

// User converters
export function toUser(api: ApiUser): User {
  return {
    id: api.id,
    email: api.email,
    displayName: api.display_name,
    createdAt: api.created_at,
  };
}

// Account converters
export function toAccount(api: ApiAccount): Account {
  return {
    id: api.id,
    name: api.name,
    ownerIds: api.owner_ids,
    createdAt: api.created_at,
    updatedAt: api.updated_at,
  };
}

export function toAccounts(api: ApiAccount[]): Account[] {
  return api.map(toAccount);
}

// Category converters
export function toCategory(api: ApiCategory): Category {
  return {
    id: api.id,
    accountId: api.account_id,
    name: api.name,
    createdAt: api.created_at,
    deletedAt: api.deleted_at,
  };
}

export function toCategories(api: ApiCategory[]): Category[] {
  return api.map(toCategory);
}

// Transaction converters
export function toTransaction(api: ApiTransaction): Transaction {
  return {
    id: api.id,
    accountId: api.account_id,
    amount: api.amount,
    categoryId: api.category_id,
    date: api.date,
    comment: api.comment,
    createdAt: api.created_at,
    updatedAt: api.updated_at,
  };
}

export function toTransactions(api: ApiTransaction[]): Transaction[] {
  return api.map(toTransaction);
}

// Paginated response converter
export function toPaginatedResponse<TApi, T>(
  api: ApiPaginatedResponse<TApi>,
  itemConverter: (item: TApi) => T
): PaginatedResponse<T> {
  return {
    items: api.items.map(itemConverter),
    nextCursor: api.next_cursor,
  };
}

// Command response converters
export function toCreateAccountResponse(api: ApiCreateAccountResponse): CreateAccountResponse {
  return {
    accountId: api.account_id,
  };
}

export function toAddCategoryResponse(api: ApiAddCategoryResponse): AddCategoryResponse {
  return {
    categoryId: api.category_id,
  };
}

export function toAddTransactionResponse(api: ApiAddTransactionResponse): AddTransactionResponse {
  return {
    transactionId: api.transaction_id,
  };
}
