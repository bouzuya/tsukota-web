// API Response types (snake_case - matches backend responses)

// User from API
export interface ApiUser {
	id: string;
	email: string;
	display_name: string;
	created_at: string;
}

// Account from API
export interface ApiAccount {
	id: string;
	name: string;
	owner_ids: string[];
	created_at: string;
	updated_at: string;
}

// Category from API
export interface ApiCategory {
	id: string;
	account_id: string;
	name: string;
	created_at: string;
	deleted_at: string | null;
}

// Transaction from API
export interface ApiTransaction {
	id: string;
	account_id: string;
	amount: string;
	category_id: string;
	date: string;
	comment: string;
	created_at: string;
	updated_at: string;
}

// Paginated response from API
export interface ApiPaginatedResponse<T> {
	items: T[];
	next_cursor: string | null;
}

// Command response types (snake_case)
export interface ApiCreateAccountResponse {
	account_id: string;
}

export interface ApiAddCategoryResponse {
	category_id: string;
}

export interface ApiAddTransactionResponse {
	transaction_id: string;
}

// RFC 9457 Problem Details (already snake_case in spec)
export interface ApiProblemDetails {
	type: string;
	title: string;
	status: number;
	detail?: string;
	instance?: string;
}

// Command bodies (snake_case - sent to backend)
export interface ApiCreateAccountCommand {
	name: string;
}

export interface ApiUpdateAccountCommand {
	account_id: string;
	name: string;
}

export interface ApiDeleteAccountCommand {
	account_id: string;
}

export interface ApiAddOwnerCommand {
	account_id: string;
	user_id: string;
}

export interface ApiRemoveOwnerCommand {
	account_id: string;
	user_id: string;
}

export interface ApiAddCategoryCommand {
	account_id: string;
	name: string;
}

export interface ApiUpdateCategoryCommand {
	account_id: string;
	category_id: string;
	name: string;
}

export interface ApiDeleteCategoryCommand {
	account_id: string;
	category_id: string;
}

export interface ApiAddTransactionCommand {
	account_id: string;
	amount: string;
	category_id: string;
	comment: string;
	date: string;
}

export interface ApiUpdateTransactionCommand {
	account_id: string;
	transaction_id: string;
	amount: string;
	category_id: string;
	comment: string;
	date: string;
}

export interface ApiDeleteTransactionCommand {
	account_id: string;
	transaction_id: string;
}
