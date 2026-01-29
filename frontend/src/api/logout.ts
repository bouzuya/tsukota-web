import { apiPost } from "./client";

export async function logout(): Promise<void> {
	await apiPost<void>("/auth/logout");
}
