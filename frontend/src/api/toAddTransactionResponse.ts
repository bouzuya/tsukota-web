import type { ApiAddTransactionResponse } from "./apiTypes";
import type { AddTransactionResponse } from "./types";

export function toAddTransactionResponse(
	api: ApiAddTransactionResponse,
): AddTransactionResponse {
	return {
		transactionId: api.transaction_id,
	};
}
