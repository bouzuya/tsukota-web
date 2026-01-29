import { apiPost } from "./client";
import type { ApiDeleteAccountCommand } from "./apiTypes";

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
