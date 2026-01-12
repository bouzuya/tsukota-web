import { atom } from 'jotai';
import type { User } from '../api/types';

// Current user state
export const currentUserAtom = atom<User | null>(null);

// Derived atom for authentication status
export const isAuthenticatedAtom = atom((get) => get(currentUserAtom) !== null);

// Loading state for auth
export const authLoadingAtom = atom<boolean>(true);
