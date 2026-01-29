import type { ApiUser } from "./apiTypes";
import type { User } from "./types";

export function toUser(api: ApiUser): User {
	return {
		id: api.id,
		email: api.email,
		displayName: api.display_name,
		createdAt: api.created_at,
	};
}
