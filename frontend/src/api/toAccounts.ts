import type { ApiAccount } from "./apiTypes";
import { toAccount } from "./toAccount";
import type { Account } from "./types";

export function toAccounts(api: ApiAccount[]): Account[] {
	return api.map(toAccount);
}
