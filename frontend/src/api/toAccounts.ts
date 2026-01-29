import type { ApiAccount } from "./apiTypes";
import type { Account } from "./types";
import { toAccount } from "./toAccount";

export function toAccounts(api: ApiAccount[]): Account[] {
	return api.map(toAccount);
}
