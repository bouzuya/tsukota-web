import { apiPost } from "./client";

export async function refreshToken(): Promise<boolean> {
	try {
		await apiPost<void>("/auth/refresh");
		return true;
	} catch {
		return false;
	}
}
