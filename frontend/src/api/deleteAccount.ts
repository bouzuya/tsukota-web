import type { ApiDeleteAccountCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface DeleteAccountCommand {
	accountId: string;
}

export async function deleteAccount(
	command: DeleteAccountCommand,
): Promise<void> {
	const apiCommand: ApiDeleteAccountCommand = {
		account_id: command.accountId,
	};
	return apiPost<void>("/commands/delete_account", apiCommand);
}
