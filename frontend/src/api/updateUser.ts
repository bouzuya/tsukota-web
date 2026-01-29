import { apiPost } from "./client";
import type { ApiUpdateUserCommand } from "./apiTypes";

export interface UpdateUserCommand {
	userId: string;
	displayName: string;
}

export async function updateUser(command: UpdateUserCommand): Promise<void> {
	const apiCommand: ApiUpdateUserCommand = {
		user_id: command.userId,
		display_name: command.displayName,
	};
	return apiPost<void>("/commands/update_user", apiCommand);
}
