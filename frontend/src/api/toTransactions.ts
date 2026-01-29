import type { ApiTransaction } from "./apiTypes";
import type { Transaction } from "./types";
import { toTransaction } from "./toTransaction";

export function toTransactions(api: ApiTransaction[]): Transaction[] {
	return api.map(toTransaction);
}
