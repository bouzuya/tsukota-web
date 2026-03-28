import type { ApiAddOwnerCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface AddOwnerCommand {
	accountId: string;
	userId: string;
}

export async function addOwner(command: AddOwnerCommand): Promise<void> {
	const apiCommand: ApiAddOwnerCommand = {
		account_id: command.accountId,
		user_id: command.userId,
	};
	return apiPost<void>("/commands/add_owner", apiCommand);
}
