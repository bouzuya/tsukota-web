import { atom } from 'jotai';
import type { User } from '../api/types';

// Manual User ID for development (bypasses OAuth)
const MANUAL_USER_ID_KEY = 'tsukota_manual_user_id';

export function getManualUserId(): string | null {
  return localStorage.getItem(MANUAL_USER_ID_KEY);
}

export function setManualUserId(userId: string | null): void {
  if (userId) {
    localStorage.setItem(MANUAL_USER_ID_KEY, userId);
  } else {
    localStorage.removeItem(MANUAL_USER_ID_KEY);
  }
}

// Current user state
export const currentUserAtom = atom<User | null>(null);

// Derived atom for authentication status
export const isAuthenticatedAtom = atom<boolean>((get) => get(currentUserAtom) !== null);

// Loading state for auth
export const authLoadingAtom = atom<boolean>(true);
