import { apiPost } from "./client";
import type { ApiCreateAccountCommand, ApiCreateAccountResponse } from "./apiTypes";
import type { CreateAccountResponse } from "./types";
import { toCreateAccountResponse } from "./toCreateAccountResponse";

export interface CreateAccountCommand {
	name: string;
}

export async function createAccount(
	command: CreateAccountCommand,
): Promise<CreateAccountResponse> {
	const apiCommand: ApiCreateAccountCommand = {
		name: command.name,
	};
	const response = await apiPost<ApiCreateAccountResponse>(
		"/commands/create_account",
		apiCommand,
	);
	return toCreateAccountResponse(response);
}
