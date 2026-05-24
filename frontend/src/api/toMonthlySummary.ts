import type { ApiMonthlySummary } from "./apiTypes";
import type { MonthlySummary } from "./types";

export function toMonthlySummary(api: ApiMonthlySummary): MonthlySummary {
	return {
		accountId: api.account_id,
		incomes: api.incomes,
		expenses: api.expenses,
	};
}
