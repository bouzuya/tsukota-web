import type { ApiPaginatedResponse } from "./apiTypes";
import type { PaginatedResponse } from "./types";

export function toPaginatedResponse<TApi, T>(
	api: ApiPaginatedResponse<TApi>,
	itemConverter: (item: TApi) => T,
): PaginatedResponse<T> {
	return {
		items: api.items.map(itemConverter),
		nextCursor: api.next_cursor,
	};
}
