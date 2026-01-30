import type { ApiAddCategoryCommand, ApiAddCategoryResponse } from "./apiTypes";
import { apiPost } from "./client";
import { toAddCategoryResponse } from "./toAddCategoryResponse";
import type { AddCategoryResponse } from "./types";

export interface AddCategoryCommand {
	accountId: string;
	name: string;
}

export async function addCategory(
	command: AddCategoryCommand,
): Promise<AddCategoryResponse> {
	const apiCommand: ApiAddCategoryCommand = {
		account_id: command.accountId,
		name: command.name,
	};
	const response = await apiPost<ApiAddCategoryResponse>(
		"/commands/add_category",
		apiCommand,
	);
	return toAddCategoryResponse(response);
}
