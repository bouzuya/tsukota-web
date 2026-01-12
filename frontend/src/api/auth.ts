import { apiGet, apiPost } from './client';
import type { User } from './types';

export function getAuthUrl(): string {
  return '/api/auth/google';
}

export async function getCurrentUser(): Promise<User | null> {
  try {
    return await apiGet<User>('/auth/me');
  } catch {
    return null;
  }
}

export async function logout(): Promise<void> {
  await apiPost<void>('/auth/logout');
}

export async function refreshToken(): Promise<boolean> {
  try {
    await apiPost<void>('/auth/refresh');
    return true;
  } catch {
    return false;
  }
}
