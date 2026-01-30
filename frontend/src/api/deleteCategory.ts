import type { ApiDeleteCategoryCommand } from "./apiTypes";
import { apiPost } from "./client";

export interface DeleteCategoryCommand {
	accountId: string;
	categoryId: string;
}

export async function deleteCategory(
	command: DeleteCategoryCommand,
): Promise<void> {
	const apiCommand: ApiDeleteCategoryCommand = {
		account_id: command.accountId,
		category_id: command.categoryId,
	};
	return apiPost<void>("/commands/delete_category", apiCommand);
}
