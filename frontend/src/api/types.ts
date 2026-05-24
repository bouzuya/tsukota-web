// Frontend types (camelCase - for internal use)

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

// Monthly summary
export interface MonthlySummary {
	accountId: string;
	/** 月別収入合計 ("YYYY-MM" -> 0 以上の金額の合計、非負値) */
	incomes: Record<string, string>;
	/** 月別支出合計 ("YYYY-MM" -> 0 未満の金額の合計、負の値) */
	expenses: Record<string, string>;
}

// Response types (camelCase)
export interface CreateAccountResponse {
	accountId: string;
}

export interface AddCategoryResponse {
	categoryId: string;
}

export interface AddTransactionResponse {
	transactionId: string;
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
