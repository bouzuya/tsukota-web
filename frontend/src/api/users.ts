import { apiGet, apiPost } from './client';
import type { User, UpdateUserCommand } from './types';

export async function getUser(userId: string): Promise<User> {
  return apiGet<User>(`/users/${userId}`);
}

export async function updateUser(command: UpdateUserCommand): Promise<void> {
  return apiPost<void>('/commands/update_user', command);
}
