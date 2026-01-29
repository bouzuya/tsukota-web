import { apiGet } from "./client";
import type { ApiAccount } from "./apiTypes";
import type { Account } from "./types";
import { toAccount } from "./toAccount";

export async function getAccount(accountId: string): Promise<Account> {
	const response = await apiGet<ApiAccount>(`/accounts/${accountId}`);
	return toAccount(response);
}
