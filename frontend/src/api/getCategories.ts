import { apiGet } from "./client";
import type { ApiCategory, ApiPaginatedResponse } from "./apiTypes";
import type { Category, PaginatedResponse } from "./types";
import { toCategory } from "./toCategory";
import { toPaginatedResponse } from "./toPaginatedResponse";

export async function getCategories(
	accountId: string,
): Promise<PaginatedResponse<Category>> {
	const response = await apiGet<ApiPaginatedResponse<ApiCategory>>(
		`/accounts/${accountId}/categories`,
	);
	return toPaginatedResponse(response, toCategory);
}
