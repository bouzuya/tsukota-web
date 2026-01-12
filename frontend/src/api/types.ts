// User
export interface User {
  id: string;
  email: string;
  displayName: string;
  createdAt: string;
}

// Account
export interface Account {
  id: string;
  name: string;
  ownerIds: string[];
  createdAt: string;
  updatedAt: string;
}

// Category
export interface Category {
  id: string;
  accountId: string;
  name: string;
  createdAt: string;
  deletedAt: string | null;
}

// Transaction
export interface Transaction {
  id: string;
  accountId: string;
  amount: string;
  categoryId: string;
  date: string;
  comment: string;
  createdAt: string;
  updatedAt: string;
}

// API Response types
export interface CreateAccountResponse {
  account_id: string;
}

export interface AddCategoryResponse {
  category_id: string;
}

export interface AddTransactionResponse {
  transaction_id: string;
}

// Paginated response
export interface PaginatedResponse<T> {
  items: T[];
  nextCursor: string | null;
}

// RFC 9457 Problem Details
export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  instance?: string;
}

// Command bodies
export interface CreateAccountCommand {
  name: string;
}

export interface UpdateAccountCommand {
  account_id: string;
  name: string;
}

export interface DeleteAccountCommand {
  account_id: string;
}

export interface AddOwnerCommand {
  account_id: string;
  user_id: string;
}

export interface RemoveOwnerCommand {
  account_id: string;
  user_id: string;
}

export interface AddCategoryCommand {
  account_id: string;
  name: string;
}

export interface UpdateCategoryCommand {
  account_id: string;
  category_id: string;
  name: string;
}

export interface DeleteCategoryCommand {
  account_id: string;
  category_id: string;
}

export interface AddTransactionCommand {
  account_id: string;
  amount: string;
  category_id: string;
  comment: string;
  date: string;
}

export interface UpdateTransactionCommand {
  account_id: string;
  transaction_id: string;
  amount: string;
  category_id: string;
  comment: string;
  date: string;
}

export interface DeleteTransactionCommand {
  account_id: string;
  transaction_id: string;
}

export interface UpdateUserCommand {
  user_id: string;
  display_name: string;
}
