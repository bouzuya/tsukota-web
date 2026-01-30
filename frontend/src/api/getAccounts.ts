import type { ApiAccount, ApiPaginatedResponse } from "./apiTypes";
import { apiGet } from "./client";
import { toAccount } from "./toAccount";
import { toPaginatedResponse } from "./toPaginatedResponse";
import type { Account, PaginatedResponse } from "./types";

export async function getAccounts(): Promise<PaginatedResponse<Account>> {
	const response = await apiGet<ApiPaginatedResponse<ApiAccount>>("/accounts");
	return toPaginatedResponse(response, toAccount);
}
