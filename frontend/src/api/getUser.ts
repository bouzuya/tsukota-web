import type { ApiUser } from "./apiTypes";
import { apiGet } from "./client";
import { toUser } from "./toUser";
import type { User } from "./types";

export async function getUser(userId: string): Promise<User> {
	const response = await apiGet<ApiUser>(`/users/${userId}`);
	return toUser(response);
}
