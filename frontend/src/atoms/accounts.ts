import { atom } from 'jotai';
import type { Account } from '../api/types';

// Accounts list
export const accountsAtom = atom<Account[]>([]);

// Loading state
export const accountsLoadingAtom = atom<boolean>(false);

// Selected account for current session
export const selectedAccountIdAtom = atom<string | null>(null);

// Derived atom to get selected account
export const selectedAccountAtom = atom((get) => {
  const accounts = get(accountsAtom);
  const selectedId = get(selectedAccountIdAtom);
  return accounts.find((a) => a.id === selectedId) ?? null;
});
