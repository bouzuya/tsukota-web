import { apiGet } from "./client";
import type { ApiTransaction, ApiPaginatedResponse } from "./apiTypes";
import type { Transaction, PaginatedResponse } from "./types";
import { toTransaction } from "./toTransaction";
import { toPaginatedResponse } from "./toPaginatedResponse";

export async function getTransactions(
	accountId: string,
	cursor?: string,
): Promise<PaginatedResponse<Transaction>> {
	const params = cursor ? `?after=${cursor}` : "";
	const response = await apiGet<ApiPaginatedResponse<ApiTransaction>>(
		`/accounts/${accountId}/transactions${params}`,
	);
	return toPaginatedResponse(response, toTransaction);
}
