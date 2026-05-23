import type { ApiMonthlySummary } from "./apiTypes";
import { apiGet } from "./client";
import { toMonthlySummary } from "./toMonthlySummary";
import type { MonthlySummary } from "./types";

export async function getMonthlySummary(
	accountId: string,
): Promise<MonthlySummary> {
	const response = await apiGet<ApiMonthlySummary>(
		`/accounts/${accountId}/stats/monthly`,
	);
	return toMonthlySummary(response);
}
