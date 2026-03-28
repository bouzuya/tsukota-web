import type { ApiRemoveOwnerCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface RemoveOwnerCommand {
	accountId: string;
	userId: string;
}

export async function removeOwner(command: RemoveOwnerCommand): Promise<void> {
	const apiCommand: ApiRemoveOwnerCommand = {
		account_id: command.accountId,
		user_id: command.userId,
	};
	return apiPost<void>("/commands/remove_owner", apiCommand);
}
