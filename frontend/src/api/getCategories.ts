import type { ApiCategory, ApiPaginatedResponse } from "./apiTypes";
import { apiGet } from "./client";
import { toCategory } from "./toCategory";
import { toPaginatedResponse } from "./toPaginatedResponse";
import type { Category, PaginatedResponse } from "./types";

export async function getCategories(
	accountId: string,
): Promise<PaginatedResponse<Category>> {
	const response = await apiGet<ApiPaginatedResponse<ApiCategory>>(
		`/accounts/${accountId}/categories`,
	);
	return toPaginatedResponse(response, toCategory);
}
