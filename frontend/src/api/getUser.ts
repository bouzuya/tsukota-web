import { apiGet } from "./client";
import type { ApiUser } from "./apiTypes";
import type { User } from "./types";
import { toUser } from "./toUser";

export async function getUser(userId: string): Promise<User> {
	const response = await apiGet<ApiUser>(`/users/${userId}`);
	return toUser(response);
}
