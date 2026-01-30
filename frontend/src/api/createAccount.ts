import type {
	ApiCreateAccountCommand,
	ApiCreateAccountResponse,
} from "./apiTypes";
import { apiPost } from "./client";
import { toCreateAccountResponse } from "./toCreateAccountResponse";
import type { CreateAccountResponse } from "./types";

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
