import { apiGet, apiPost } from "./client";
import type {
	ApiCategory,
	ApiAddCategoryCommand,
	ApiAddCategoryResponse,
	ApiUpdateCategoryCommand,
	ApiDeleteCategoryCommand,
	ApiPaginatedResponse,
} from "./apiTypes";
import type { Category, AddCategoryResponse, PaginatedResponse } from "./types";
import {
	toAddCategoryResponse,
	toPaginatedResponse,
	toCategory,
} from "./converters";

export async function getCategories(
	accountId: string,
): Promise<PaginatedResponse<Category>> {
	const response = await apiGet<ApiPaginatedResponse<ApiCategory>>(
		`/accounts/${accountId}/categories`,
	);
	return toPaginatedResponse(response, toCategory);
}

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
