import type { ApiUpdateCategoryCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface UpdateCategoryCommand {
	accountId: string;
	categoryId: string;
	name: string;
}

export async function updateCategory(
	command: UpdateCategoryCommand,
): Promise<void> {
	const apiCommand: ApiUpdateCategoryCommand = {
		account_id: command.accountId,
		category_id: command.categoryId,
		name: command.name,
	};
	return apiPost<void>("/commands/update_category", apiCommand);
}
