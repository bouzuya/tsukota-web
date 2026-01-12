import { apiGet, apiPost } from './client';
import type {
  Category,
  AddCategoryCommand,
  AddCategoryResponse,
  UpdateCategoryCommand,
  DeleteCategoryCommand,
} from './types';

export async function getCategories(accountId: string): Promise<Category[]> {
  return apiGet<Category[]>(`/accounts/${accountId}/categories`);
}

export async function addCategory(
  command: AddCategoryCommand
): Promise<AddCategoryResponse> {
  return apiPost<AddCategoryResponse>('/commands/add_category', command);
}

export async function updateCategory(command: UpdateCategoryCommand): Promise<void> {
  return apiPost<void>('/commands/update_category', command);
}

export async function deleteCategory(command: DeleteCategoryCommand): Promise<void> {
  return apiPost<void>('/commands/delete_category', command);
}
