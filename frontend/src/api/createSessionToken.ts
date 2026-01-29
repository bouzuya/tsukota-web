import { apiPost } from "./client";
import type {
	ApiCreateSessionTokenCommand,
	ApiCreateSessionTokenResponse,
} from "./apiTypes";

export interface CreateSessionTokenParams {
	deviceId: string;
	deviceSecret: string;
}

export async function createSessionToken(
	params: CreateSessionTokenParams,
): Promise<string> {
	const response = await apiPost<
		ApiCreateSessionTokenResponse,
		ApiCreateSessionTokenCommand
	>("/commands/create_session_token", {
		device_id: params.deviceId,
		device_secret: params.deviceSecret,
	});
	return response.session_token;
}
