import type { ApiAddCategoryResponse } from "./apiTypes";
import type { AddCategoryResponse } from "./types";

export function toAddCategoryResponse(
	api: ApiAddCategoryResponse,
): AddCategoryResponse {
	return {
		categoryId: api.category_id,
	};
}
