import type { ApiTransaction } from "./apiTypes";
import type { Transaction } from "./types";

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
