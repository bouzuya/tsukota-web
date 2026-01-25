import { apiGet, apiPost } from "./client";
import type {
	ApiCreateCustomTokenCommand,
	ApiCreateCustomTokenResponse,
	ApiUser,
} from "./apiTypes";
import type { User } from "./types";
import { toUser } from "./converters";

export interface CreateCustomTokenParams {
	deviceId: string;
	deviceSecret: string;
}

export async function createCustomToken(
	params: CreateCustomTokenParams,
): Promise<string> {
	const response = await apiPost<
		ApiCreateCustomTokenResponse,
		ApiCreateCustomTokenCommand
	>("/commands/create_custom_token", {
		device_id: params.deviceId,
		device_secret: params.deviceSecret,
	});
	return response.custom_token;
}

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
