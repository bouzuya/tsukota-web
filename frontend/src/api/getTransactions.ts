import type { ApiPaginatedResponse, ApiTransaction } from "./apiTypes";
import { apiGet } from "./client";
import { toPaginatedResponse } from "./toPaginatedResponse";
import { toTransaction } from "./toTransaction";
import type { PaginatedResponse, Transaction } from "./types";

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
