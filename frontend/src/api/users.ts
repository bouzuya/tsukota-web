import { apiGet, apiPost } from './client';
import type { ApiUser, ApiUpdateUserCommand } from './apiTypes';
import type { User } from './types';
import { toUser } from './converters';

export async function getUser(userId: string): Promise<User> {
  const response = await apiGet<ApiUser>(`/users/${userId}`);
  return toUser(response);
}

export interface UpdateUserCommand {
  userId: string;
  displayName: string;
}

export async function updateUser(command: UpdateUserCommand): Promise<void> {
  const apiCommand: ApiUpdateUserCommand = {
    user_id: command.userId,
    display_name: command.displayName,
  };
  return apiPost<void>('/commands/update_user', apiCommand);
}
