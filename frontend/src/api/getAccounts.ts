import { apiGet } from "./client";
import type { ApiAccount, ApiPaginatedResponse } from "./apiTypes";
import type { Account, PaginatedResponse } from "./types";
import { toAccount } from "./toAccount";
import { toPaginatedResponse } from "./toPaginatedResponse";

export async function getAccounts(): Promise<PaginatedResponse<Account>> {
	const response = await apiGet<ApiPaginatedResponse<ApiAccount>>("/accounts");
	return toPaginatedResponse(response, toAccount);
}
