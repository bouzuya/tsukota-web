import { apiGet } from "./client";
import type { ApiUser } from "./apiTypes";
import type { User } from "./types";
import { toUser } from "./toUser";

export async function getCurrentUser(): Promise<User | null> {
	try {
		const response = await apiGet<ApiUser>("/auth/me");
		return toUser(response);
	} catch {
		return null;
	}
}
