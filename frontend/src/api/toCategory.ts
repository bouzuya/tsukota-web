import type { ApiCategory } from "./apiTypes";
import type { Category } from "./types";

export function toCategory(api: ApiCategory): Category {
	return {
		id: api.id,
		accountId: api.account_id,
		name: api.name,
		createdAt: api.created_at,
		deletedAt: api.deleted_at,
	};
}
