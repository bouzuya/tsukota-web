import type { ApiAccount } from "./apiTypes";
import { apiGet } from "./client";
import { toAccount } from "./toAccount";
import type { Account } from "./types";

export async function getAccount(accountId: string): Promise<Account> {
	const response = await apiGet<ApiAccount>(`/accounts/${accountId}`);
	return toAccount(response);
}
