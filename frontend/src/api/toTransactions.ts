import type { ApiTransaction } from "./apiTypes";
import { toTransaction } from "./toTransaction";
import type { Transaction } from "./types";

export function toTransactions(api: ApiTransaction[]): Transaction[] {
	return api.map(toTransaction);
}
