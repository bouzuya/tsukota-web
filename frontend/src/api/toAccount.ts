import type { ApiAccount } from "./apiTypes";
import type { Account } from "./types";

export function toAccount(api: ApiAccount): Account {
	return {
		id: api.id,
		name: api.name,
		ownerIds: api.owner_ids,
		createdAt: api.created_at,
		updatedAt: api.updated_at,
	};
}
