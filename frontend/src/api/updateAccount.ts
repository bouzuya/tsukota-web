import type { ApiUpdateAccountCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface UpdateAccountCommand {
	accountId: string;
	name: string;
}

export async function updateAccount(
	command: UpdateAccountCommand,
): Promise<void> {
	const apiCommand: ApiUpdateAccountCommand = {
		account_id: command.accountId,
		name: command.name,
	};
	return apiPost<void>("/commands/update_account", apiCommand);
}
