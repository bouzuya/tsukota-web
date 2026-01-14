import { apiGet, apiPost } from "./client";
import type { ApiUser } from "./apiTypes";
import type { User } from "./types";
import { toUser } from "./converters";

export function getAuthUrl(): string {
	return "/api/auth/google";
}

export async function getCurrentUser(): Promise<User | null> {
	try {
		const response = await apiGet<ApiUser>("/auth/me");
		return toUser(response);
	} catch {
		return null;
	}
}

export async function logout(): Promise<void> {
	await apiPost<void>("/auth/logout");
}

export async function refreshToken(): Promise<boolean> {
	try {
		await apiPost<void>("/auth/refresh");
		return true;
	} catch {
		return false;
	}
}
